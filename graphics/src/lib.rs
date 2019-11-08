use std::sync::Arc;
use vulkano::{
    buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        AutoCommandBuffer,
        AutoCommandBufferBuilder,
        CommandBuffer,
        DynamicState,
    },
    device::{Device, DeviceExtensions, Queue},
    format::ClearValue,
    framebuffer::{
        Framebuffer,
        FramebufferAbstract,
        RenderPass,
        RenderPassAbstract,
        Subpass,
    },
    image::SwapchainImage,
    instance::{Instance, PhysicalDevice},
    pipeline::{viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract},
    swapchain::{
        self,
        acquire_next_image,
        AcquireError,
        ColorSpace,
        PresentMode,
        Surface,
        SurfaceTransform,
        Swapchain,
        SwapchainAcquireFuture,
        SwapchainCreationError,
    },
    sync,
    sync::{FlushError, GpuFuture},
};
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Window, WindowBuilder};

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450
layout(location = 0) in vec2 position;
void main() {
gl_Position = vec4(position, 0.0, 1.0);
}"
    }
}
mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450
layout(location = 0) out vec4 f_color;
void main() {
f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"
    }
}
pub struct Graphics {
    instance:           Arc<Instance>,
    surface:            Arc<Surface<Window>>,
    device:             Arc<Device>,
    queue:              Arc<Queue>,
    render_pass:        Arc<dyn RenderPassAbstract + Sync + Send>,
    framebuffers:       Vec<Arc<(dyn FramebufferAbstract + Send + Sync)>>,
    pipeline:           Arc<dyn GraphicsPipelineAbstract + Sync + Send>,
    swapchain:          Arc<Swapchain<Window>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swapchain: bool,
    vertex_buffer:      Vec<Arc<dyn BufferAccess + Send + Sync + 'static>>,
    dimensions:         [u32; 2],
    images:             Vec<Arc<vulkano::image::SwapchainImage<winit::Window>>>,
    dynamic_state:      DynamicState,
    acquire_future:     SwapchainAcquireFuture<Window>,
    command_buffer:     Arc<AutoCommandBuffer>,
}
#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
impl Graphics {
    pub fn new(events_loop: &EventsLoop) -> Self {
        let instance = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).unwrap()
        };
        let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
        let surface = WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();
        let surface = WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();
        let queue_family = physical
            .queue_families()
            .find(|&q| {
                q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
            })
            .unwrap();
        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (device, mut queues) = Device::new(
            physical,
            physical.supported_features(),
            &device_ext,
            [(queue_family, 0.5)].iter().cloned(),
        )
        .unwrap();
        let vs = vs::Shader::load(device.clone()).unwrap();
        let fs = fs::Shader::load(device.clone()).unwrap();
        let queue = queues.next().unwrap();
        let initial_dimensions = {
            //  returns early here, thanks mr ?
            let logical_dimensions = surface
                .window()
                .get_inner_size()
                .expect("Could not get window dimensions");
            let dimensions: (u32, u32) = logical_dimensions
                .to_physical(surface.window().get_hidpi_factor())
                .into();
            [dimensions.0, dimensions.1]
        };
        let mut recreate_swapchain = false;
        let (mut swapchain, images) = {
            let caps = surface.capabilities(physical).unwrap();
            let usage = caps.supported_usage_flags;
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;

            Swapchain::new(
                device.clone(),
                surface.clone(),
                caps.min_image_count,
                format,
                initial_dimensions,
                1,
                usage,
                &queue,
                SurfaceTransform::Identity,
                alpha,
                PresentMode::Fifo,
                true,
                ColorSpace::SrgbNonLinear,
            )
            .unwrap()
        };
        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .unwrap(),
        );
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
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
        let mut framebuffers = window_size_dependent_setup(
            &images,
            render_pass.clone(),
            &mut dynamic_state,
        );
        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(err) => panic!("{:?}", err),
            };
        let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];
        let vertex_buffer = {
            vulkano::impl_vertex!(Vertex, position);

            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                [
                    Vertex {
                        position: [-0.5, -0.25],
                    },
                    Vertex {
                        position: [0.0, 0.5],
                    },
                    Vertex {
                        position: [0.25, -0.1],
                    },
                ]
                .iter()
                .cloned(),
            )
            .unwrap()
        };
        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            device.clone(),
            queue.family(),
        )
        .unwrap()
        .begin_render_pass(framebuffers[image_num].clone(), false, clear_values)
        .unwrap()
        .draw(
            pipeline.clone(),
            &dynamic_state,
            vertex_buffer.clone(),
            (),
            (),
        )
        .unwrap()
        .end_render_pass()
        .unwrap()
        .build()
        .unwrap();
        let previous_frame_end = Some(Self::create_sync_objects(&device));
        Self {
            instance,
            surface,
            device,
            queue,
            pipeline,
            swapchain,
            render_pass,
            framebuffers,
            command_buffer: Arc::new(command_buffer),
            vertex_buffer: vec![vertex_buffer],
            recreate_swapchain,
            dimensions: initial_dimensions,
            dynamic_state,
            acquire_future,
            previous_frame_end,
            images,
        }
    }

    pub fn render(
        &mut self,
        remake_swapchain: bool,
    ) {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();
        if remake_swapchain {
            self.recreate_swapchain();
        }
        self.draw_frame();
    }

    fn recreate_swapchain(&mut self) {
        if self.recreate_swapchain {
            self.dimensions = {
                let dd: (u32, u32) = self
                    .surface
                    .window()
                    .get_inner_size()
                    .expect("Dimensions are wrong.")
                    .to_physical(self.surface.window().get_hidpi_factor())
                    .into();
                [dd.0, dd.1]
            };
            println!("{:?}", self.dimensions);
            let (new_swapchain, new_images) = self
                .swapchain
                .recreate_with_dimension(self.dimensions)
                .expect("Unsupported dimensions?");
            self.swapchain = new_swapchain;
            self.framebuffers = window_size_dependent_setup(
                &new_images,
                self.render_pass.clone(),
                &mut self.dynamic_state,
            );
        }
        self.recreate_swapchain = false;
    }

    fn draw_frame(&mut self) {
        let (image_index, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(err) => panic!("{:?}", err),
            };
        println!("acquire_future");
        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), self.command_buffer.clone())
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                self.swapchain.clone(),
                image_index,
            )
            .then_signal_fence_and_flush();
        // .unwrap();
        println!("future.unwrap()");
        match future {
            Ok(future) => {
                // This wait is required when using NVIDIA or running on macOS.
                // See https://github.com/vulkano-rs/vulkano/issues/1247
                future.wait(None).unwrap();
                self.previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(sync::FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end =
                    Some(Box::new(vulkano::sync::now(self.device.clone()))
                        as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end =
                    Some(Box::new(vulkano::sync::now(self.device.clone()))
                        as Box<_>);
            }
        }
        // future.wait(None).unwrap();
        println!("done draw_frame");
    }

    pub fn window(&self) -> &Window { self.surface.window() }

    // pub fn recreate_swapchain(&mut self) { self.recreate_swapchain = true; }

    fn create_sync_objects(device: &Arc<Device>) -> Box<dyn GpuFuture> {
        Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>
    }
}
fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
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
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
