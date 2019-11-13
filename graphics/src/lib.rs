use common::{Entity, Quaternion};
use num_traits::{Float, One, Zero};
use std::sync::Arc;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::{
    buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        AutoCommandBuffer, AutoCommandBufferBuilder, CommandBuffer, DynamicState,
    },
    device::{Device, DeviceExtensions, Queue},
    format::{ClearValue, Format::D16Unorm},
    framebuffer::{
        Framebuffer, FramebufferAbstract, RenderPass, RenderPassAbstract, Subpass,
    },
    image::{attachment::AttachmentImage, SwapchainImage},
    instance::{Instance, PhysicalDevice},
    pipeline::{
        viewport::Viewport, ComputePipeline, GraphicsPipeline,
        GraphicsPipelineAbstract,
    },
    swapchain::{
        self, acquire_next_image, AcquireError, ColorSpace, PresentMode, Surface,
        SurfaceTransform, Swapchain, SwapchainAcquireFuture,
        SwapchainCreationError,
    },
    sync,
    sync::{FlushError, GpuFuture},
};
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Window, WindowBuilder};

mod cs {
    vulkano_shaders::shader! {ty: "compute", path: "shaders/comp.glsl"}
}
mod vs {
    vulkano_shaders::shader! {ty: "vertex", path:"shaders/vert.glsl"}
}
mod fs {
    vulkano_shaders::shader! {ty: "fragment", path:"shaders/frag.glsl"}
}
#[derive(Default, Debug, Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    orient: Quaternion<f32>,
}
pub struct Graphics {
    instance: Arc<Instance>,
    surface: Arc<Surface<Window>>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    render_pass: Arc<dyn RenderPassAbstract + Sync + Send>,
    framebuffers: Vec<Arc<(dyn FramebufferAbstract + Send + Sync)>>,
    swapchain: Arc<Swapchain<Window>>,
    previous_frame_end: Option<Box<dyn GpuFuture + Send + Sync>>,
    recreate_swapchain: bool,
    dimensions: [u32; 2],
    images: Vec<Arc<vulkano::image::SwapchainImage<winit::Window>>>,
    dynamic_state: DynamicState,
    /* pipeline:           Arc<dyn GraphicsPipelineAbstract + Sync + Send>,
     * vertex_buffer:      Vec<Arc<dyn BufferAccess + Send + Sync>>,
     * acquire_future:     SwapchainAcquireFuture<Window>,
     * command_buffer:     Arc<AutoCommandBuffer>, */
}

impl Graphics {
    pub fn new(events_loop: &EventsLoop) -> Self {
        let instance = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).unwrap()
        };
        let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
        let surface = WindowBuilder::new()
            .with_title("CA01")
            .with_decorations(true)
            .with_transparency(true)
            .with_fullscreen(Some(
                events_loop.get_available_monitors().nth(0).unwrap(),
            ))
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
        let queue = queues.next().unwrap();
        let dimensions = {
            let logical_dimensions = surface
                .window()
                .get_inner_size()
                .expect("Could not get window dimensions.");
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
                dimensions,
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
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };
        let depth_buffer = AttachmentImage::transient(
            queue.device().clone(),
            dimensions,
            D16Unorm,
        )
        .unwrap();
        let data_buffer = data_buffer_setup(device.clone());
        let mut framebuffers = window_size_dependent_setup(
            &images,
            render_pass.clone(),
            &mut dynamic_state,
            device.clone(),
        );
        let previous_frame_end = Some(Self::create_sync_objects(&device));
        Self {
            instance,
            surface,
            device,
            queue,
            swapchain,
            render_pass,
            framebuffers,
            recreate_swapchain,
            dimensions,
            dynamic_state,
            previous_frame_end,
            images,
        }
    }

    pub fn render(
        &mut self,
        remake_swap: bool,
        mouse: [f64; 2],
    ) {
        self.recreate_swapchain = remake_swap;
        self.remake_swapchain();
        // let cs = fs::Shader::load(self.device.clone()).unwrap();
        let vs = vs::Shader::load(self.device.clone()).unwrap();
        let fs = fs::Shader::load(self.device.clone()).unwrap();
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .depth_stencil_simple_depth()
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(self.render_pass.clone(), 0).unwrap())
                .build(self.device.clone())
                .unwrap(),
        );
        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(err) => panic!("{:?}", err),
            };
        let clear_values = vec![[0.0, 0.0, 0.0, 0.0].into(), 1f32.into()];
        let cube = {
            let mut x = [Vertex {
                position: [0.0f32, 0.0f32, 0.0f32],
                orient: Quaternion::<f32>::one(),
            }; 3];
            let e = Entity::<f32>::new();
            let z = [
                [0.8, 0.8, 0.8],
                [-0.8, 0.8, 0.1],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, -0.5],
                [-0.5, -0.5, -0.5],
            ];
            for i in 0..3 {
                x[i].position.copy_from_slice(&z[i][0..3]);
                x[i].orient.val.copy_from_slice(&e.orient.val);
            }
            x[2].position[0..2].copy_from_slice(
                &vec![
                    mouse[0] as f32 / self.dimensions[0] as f32 * 2.0 - 1.0f32,
                    mouse[1] as f32 / self.dimensions[1] as f32 * 2.0 - 1.0f32,
                ][0..2],
            );
            x
        };
        // let compute_pipeline = Arc::new(
        //     ComputePipeline::new(
        //         self.device.clone(),
        //         &cs.main_entry_point(),
        //         &(),
        //     )
        //     .expect("failed to create compute pipeline"),
        // );
        //
        // let data_buffer = CpuAccessibleBuffer::from_iter(
        //     self.device.clone(),
        //     BufferUsage::all(),
        //     {
        //         let mut x = [Vertex {
        //             position: [0.0f32, 0.0f32, 0.0f32],
        //             orient: Quaternion::<f32>::one(),
        //         }; 8];
        //         let e = Entity::<f32>::new();
        //         let z = [
        //             [0.8, 0.8, 0.8],
        //             [-0.8, 0.8, 0.1],
        //             [0.5, -0.5, 0.5],
        //             [0.5, 0.5, -0.5],
        //             [-0.5, 0.5, -0.5],
        //             [-0.5, -0.5, 0.5],
        //             [0.5, -0.5, -0.5],
        //             [-0.5, -0.5, -0.5],
        //         ];
        //         for i in 0..8 {
        //             x[i].position.copy_from_slice(&z[i][0..3]);
        //             x[i].orient.val.copy_from_slice(&e.orient.val);
        //         }
        //         x
        //     }
        //     .iter()
        //     .cloned(),
        // )
        // .expect("Failed to create data_buffer");
        // let set = Arc::new(
        //     PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
        //         .add_buffer(data_buffer.clone())
        //         .unwrap()
        //         .build()
        //         .unwrap(),
        // );
        //
        // let command_buffer = AutoCommandBufferBuilder::new(
        //     self.device.clone(),
        //     self.queue.family(),
        // )
        // .unwrap()
        // .dispatch([2, 1, 1], compute_pipeline.clone(), set.clone(), ())
        // .unwrap()
        // .build()
        // .unwrap();
        // let finished = command_buffer.execute(self.queue.clone()).unwrap();
        // finished
        //     .then_signal_fence_and_flush()
        //     .unwrap()
        //     .wait(None)
        //     .unwrap();
        // let content = data_buffer.read().unwrap();
        // for (n, val) in content.iter().enumerate() {
        //     println!("{}{:?}", n, val);
        // }
        let vertex_buffer = {
            vulkano::impl_vertex!(Vertex, position, orient);
            CpuAccessibleBuffer::from_iter(
                self.device.clone(),
                BufferUsage::all(),
                cube.iter().cloned(),
            )
            .unwrap()
        };
        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap()
        .begin_render_pass(
            self.framebuffers[image_num].clone(),
            false,
            clear_values,
        )
        .unwrap()
        .draw(pipeline.clone(), &self.dynamic_state, vertex_buffer, (), ())
        .unwrap()
        .end_render_pass()
        .unwrap()
        .build()
        .unwrap();
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();
        self.draw_frame(command_buffer, acquire_future, image_num);
    }

    fn draw_frame(
        &mut self,
        command_buffer: AutoCommandBuffer,
        acquire_future: SwapchainAcquireFuture<Window>,
        image_num: usize,
    ) {
        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                self.swapchain.clone(),
                image_num,
            )
            .then_signal_fence_and_flush();
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
    }

    pub fn window(&self) -> &Window {
        self.surface.window()
    }

    // pub fn recreate_swapchain(&mut self) { self.recreate_swapchain = true; }

    fn remake_swapchain(&mut self) {
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
            let (new_swapchain, new_images) = self
                .swapchain
                .recreate_with_dimension(self.dimensions)
                .expect("Unsupported dimensions?");
            self.swapchain = new_swapchain;
            self.images = new_images;
            self.framebuffers = window_size_dependent_setup(
                &self.images,
                self.render_pass.clone(),
                &mut self.dynamic_state,
                self.device.clone(),
            );
        }
        self.recreate_swapchain = false;
    }

    fn create_sync_objects(
        device: &Arc<Device>
    ) -> Box<dyn GpuFuture + Send + Sync> {
        Box::new(sync::now(device.clone())) as Box<dyn GpuFuture + Send + Sync>
    }
}
fn data_buffer_setup(device: Arc<Device>) {
    let e = Entity::<f32>::new();
    let q = Quaternion::<f32>::one();
    CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        e.model.mesh.iter().copied(),
    )
    .expect("failed to create buffer");
}
fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
    device: Arc<Device>,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);

    images
        .iter()
        .map(|image| {
            let depth_buffer =
                AttachmentImage::transient(device.clone(), dimensions, D16Unorm)
                    .unwrap();
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
