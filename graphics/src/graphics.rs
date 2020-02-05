use crate::Graphics;
use common::managers::AssetManager;
use vulkano::command_buffer::DynamicState;

use std::sync::Arc;

use crate::Renderer;
use vulkano::{
    self,
    device::{Device, DeviceExtensions},
    format::Format::D16Unorm,
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract},
    image::{attachment::AttachmentImage, SwapchainImage},
    instance::{Instance, PhysicalDevice},
    pipeline::viewport::Viewport,
    swapchain::{
        ColorSpace,
        FullscreenExclusive,
        PresentMode,
        SurfaceTransform,
        Swapchain,
    },
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

impl Graphics {
    pub fn new(eventl: &EventLoop<()>) -> Self {
        let instance = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).unwrap()
        };
        let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
        let surface = WindowBuilder::new()
            .with_title("CA01")
            .with_decorations(true)
            .with_transparent(true)
            // .with_fullscreen(Some(
            //     events_loop.get_available_monitors().nth(0).unwrap(),
            // ))
            .build_vk_surface(&eventl, instance.clone())
            .unwrap();
        let queue_family = physical
            .queue_families()
            .find(|&q| {
                q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
            })
            .unwrap();
        let (device, mut queues) = Device::new(
            physical,
            physical.supported_features(),
            &DeviceExtensions {
                khr_swapchain: true,
                khr_storage_buffer_storage_class: true,
                ..DeviceExtensions::none()
            },
            [(queue_family, 0.5)].iter().cloned(),
        )
        .unwrap();
        let queue = queues.next().unwrap();
        let dimensions = {
            let logical_dimensions = surface.window().inner_size();
            let dimensions: (u32, u32) = logical_dimensions.into();
            [dimensions.0, dimensions.1]
        };
        let recreate_swapchain = false;
        let (swapchain, images) = {
            let caps = surface.capabilities(physical).unwrap();
            let usage = caps.supported_usage_flags;
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;

            Swapchain::new(
                device.clone(),
                surface.clone(),
                caps.min_image_count,
                format,
                dimensions,
                1,
                usage,
                &queue,
                SurfaceTransform::Identity,
                alpha,
                PresentMode::Fifo,
                FullscreenExclusive::Default,
                true,
                ColorSpace::SrgbNonLinear,
            )
            .unwrap()
        };
        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    },

                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: D16Unorm,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {depth}
                }
            )
            .unwrap(),
        );
        let mut dynamic_state = DynamicState {
            line_width:   None,
            viewports:    None,
            scissors:     None,
            compare_mask: None,
            write_mask:   None,
            reference:    None,
        };
        let depth_buffer = AttachmentImage::transient(
            queue.device().clone(),
            dimensions,
            D16Unorm,
        )
        .unwrap();

        let framebuffers = window_size_dependent_setup(
            &images,
            render_pass.clone(),
            &mut dynamic_state,
            depth_buffer.clone(),
        );

        let renderer = Renderer::new(
            device.clone(),
            queue.clone(),
            &images,
            render_pass.clone(),
        );
        Self {
            // instance,
            surface,
            device,
            queue,
            swapchain,
            render_pass,
            framebuffers,
            recreate_swapchain,
            dimensions,
            dynamic_state,
            images,
            depth_buffer,
            renderer,
        }
    }

    pub fn render(
        &mut self,
        am: &AssetManager,
        mouse: [f64; 2],
    ) {
        self.remake_swapchain();

        self.renderer.compute(
            self.device.clone(),
            self.queue.clone(),
            self.dimensions,
            am,
            mouse,
            &self.images,
        );
        self.recreate_swapchain = self.renderer.render(
            self.queue.clone(),
            self.device.clone(),
            self.swapchain.clone(),
            self.framebuffers.clone(),
            &mut self.dynamic_state,
        );
    }

    pub fn window(&self) -> &Window { self.surface.window() }

    // pub fn recreate_swapchain(&mut self) { self.recreate_swapchain = true; }

    fn remake_swapchain(&mut self) {
        if self.recreate_swapchain {
            self.dimensions = {
                let d: (u32, u32) = self.surface.window().inner_size().into();
                [d.0, d.1]
            };
            let (new_swapchain, new_images) = self
                .swapchain
                .recreate_with_dimensions(self.dimensions)
                .expect("Unsupported dimensions?");
            self.swapchain = new_swapchain;
            self.images = new_images;
            self.depth_buffer = AttachmentImage::transient(
                self.queue.device().clone(),
                self.dimensions,
                D16Unorm,
            )
            .unwrap();
            self.framebuffers = window_size_dependent_setup(
                &self.images,
                self.render_pass.clone(),
                &mut self.dynamic_state,
                self.depth_buffer.clone(),
            );
        }
        self.recreate_swapchain = false;
    }

    pub fn recreate_swapchain(&mut self) { self.recreate_swapchain = true; }
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
    depth_buffer: Arc<AttachmentImage>,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();
    let viewport = Viewport {
        origin:      [0.0, 0.0],
        dimensions:  [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);
    images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .add(depth_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
