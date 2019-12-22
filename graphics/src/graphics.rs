use crate::Graphics;
use common::{managers::AssetManager, Entity, Quaternion};

use std::sync::Arc;
use vulkano::buffer::BufferAccess;

use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        AutoCommandBuffer,
        AutoCommandBufferBuilder,
        CommandBuffer,
        DynamicState,
    },
    descriptor::{
        descriptor_set::PersistentDescriptorSet,
        pipeline_layout::PipelineLayoutAbstract,
    },
    device::{Device, DeviceExtensions},
    format::Format::D16Unorm,
    framebuffer::{
        Framebuffer,
        FramebufferAbstract,
        RenderPassAbstract,
        Subpass,
    },
    image::{attachment::AttachmentImage, SwapchainImage},
    instance::{Instance, PhysicalDevice},
    pipeline::{
        vertex::TwoBuffersDefinition,
        viewport::Viewport,
        ComputePipeline,
        GraphicsPipeline,
    },
    swapchain::{
        self,
        AcquireError,
        ColorSpace,
        PresentMode,
        Surface,
        SurfaceTransform,
        Swapchain,
        SwapchainAcquireFuture,
    },
    sync,
    sync::GpuFuture,
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
// Used to force recompilation of shader change
#[allow(dead_code)]
const SHADER1: &str = include_str!("../shaders/comp.glsl");
#[allow(dead_code)]
const SHADER2: &str = include_str!("../shaders/vert.glsl");
#[allow(dead_code)]
const SHADER3: &str = include_str!("../shaders/frag.glsl");

#[derive(Default, Debug, Clone, Copy)]
struct Vertex {
    position: [f32; 4],
    orient:   [f32; 4],
    normals:  [f32; 4],
}
#[derive(Default, Debug, Clone)]
struct Normal {
    normal: [f32; 4],
}
vulkano::impl_vertex!(Normal, normal);
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
            // .with_fullscreen(Some(
            //     events_loop.get_available_monitors().nth(0).unwrap(),
            // ))
            .build_vk_surface(&events_loop, instance.clone())
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
            let logical_dimensions = surface
                .window()
                .get_inner_size()
                .expect("Could not get window dimensions.");
            let dimensions: (u32, u32) = logical_dimensions
                .to_physical(surface.window().get_hidpi_factor())
                .into();
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
        // let data_buffer = data_buffer_setup(device.clone());
        let framebuffers = window_size_dependent_setup(
            &images,
            render_pass.clone(),
            &mut dynamic_state,
            device.clone(),
            depth_buffer.clone(),
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
            depth_buffer,
        }
    }

    pub fn render(
        &mut self,
        remake_swap: bool,
        mouse: [f64; 2],
    ) {
        self.recreate_swapchain = remake_swap;
        self.remake_swapchain();
        let cs = cs::Shader::load(self.device.clone()).unwrap();
        let vs = vs::Shader::load(self.device.clone()).unwrap();
        let fs = fs::Shader::load(self.device.clone()).unwrap();
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                // .vertex_input_single_buffer::<Vertex>()
                .vertex_input(TwoBuffersDefinition::<Vertex,Normal>::new())
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
        let clear_values = vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()];
        let compute_pipeline = Arc::new(
            ComputePipeline::new(
                self.device.clone(),
                &cs.main_entry_point(),
                &(),
            )
            .expect("failed to create compute pipeline"),
        );

        let data_buffer = {
            vulkano::impl_vertex!(Vertex, position, orient, normals);
            CpuAccessibleBuffer::from_iter(
                self.device.clone(),
                BufferUsage::all(),
                {
                    let mut e = AssetManager::new().load::<f32>("teapot");
                    e.model.scale = 0.25;
                    let mut x = vec![];
                    for i in 0..e.len {
                        x.push(Vertex {
                            position: (Quaternion::new([
                                e.model.positions[i][0],
                                e.model.positions[i][1],
                                e.model.positions[i][2],
                                0.0,
                            ]) * e.model.scale)
                                .val,
                            orient:   Quaternion::new([
                                (mouse[1] as f32 / self.dimensions[1] as f32)
                                    .cos(),
                                -(mouse[0] as f32 / self.dimensions[0] as f32)
                                    .sin(),
                                -(mouse[0] as f32 / self.dimensions[0] as f32)
                                    .sin(),
                                (mouse[1] as f32 / self.dimensions[1] as f32)
                                    .sin(),
                            ])
                            .u_mut()
                            .val,
                            normals:  [0.0f32; 4],
                        });
                    }
                    x
                }
                .iter()
                .cloned(),
            )
            .expect("Failed to create data_buffer")
        };
        let uniform_buffer = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            vec![
                self.dimensions[0] as f32 / 2560.0,
                self.dimensions[1] as f32 / 1440.0,
            ]
            .iter()
            .cloned(),
        )
        .expect("Failed to create data_buffer");

        let set = Arc::new(
            PersistentDescriptorSet::start(
                compute_pipeline
                    .layout()
                    .descriptor_set_layout(0)
                    .unwrap()
                    .clone(),
            )
            .add_buffer(data_buffer.clone())
            .unwrap()
            .add_buffer(uniform_buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
        );
        let command_buffer = AutoCommandBufferBuilder::new(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap()
        .dispatch([1024, 1, 1], compute_pipeline.clone(), set.clone(), ())
        .unwrap()
        .build()
        .unwrap();
        command_buffer
            .execute(self.queue.clone())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
        // let content = data_buffer.read().unwrap();
        // for (n, val) in content.iter().enumerate() {
        //     println!("{} {:?}", n, val);
        // }
        let vertex_buffer: Arc<dyn BufferAccess + Send + Sync> = {
            CpuAccessibleBuffer::from_iter(
                self.device.clone(),
                BufferUsage::all(),
                data_buffer.read().unwrap().iter().cloned(),
            )
            .unwrap()
        };
        let (index_buffer, normals_buffer) = {
            let mut e = AssetManager::new().load::<f32>("teapot");
            e.model.scale = 0.5;
            (
                CpuAccessibleBuffer::from_iter(
                    self.device.clone(),
                    BufferUsage::index_buffer(),
                    e.model.indices.iter().cloned(),
                )
                .expect("Failed to create index_buffer"),
                CpuAccessibleBuffer::from_iter(
                    self.device.clone(),
                    BufferUsage::vertex_buffer(),
                    {
                        let mut x = vec![];
                        for i in 0..e.len {
                            x.push(Vertex {
                                position: (Quaternion::new([
                                    e.model.positions[i][0],
                                    e.model.positions[i][1],
                                    e.model.positions[i][2],
                                    0.0,
                                ]) * e.model.scale)
                                    .val,
                                orient:   [0.0; 4],
                                normals:  [
                                    e.model.normals[i][0],
                                    e.model.normals[i][1],
                                    e.model.normals[i][2],
                                    0.0,
                                ],
                            });
                        }
                        x
                    }
                    .iter()
                    .cloned(),
                )
                .expect("Failed to create normals_buffer"),
            )
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
        .draw_indexed(
            pipeline.clone(),
            &self.dynamic_state,
            vec![vertex_buffer.clone(), normals_buffer.clone()],
            index_buffer.clone(),
            (),
            (),
        )
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

    pub fn window(&self) -> &Window { self.surface.window() }

    // pub fn recreate_swapchain(&mut self) { self.recreate_swapchain = true; }

    fn remake_swapchain(&mut self) {
        if self.recreate_swapchain {
            self.dimensions = {
                let d: (u32, u32) = self
                    .surface
                    .window()
                    .get_inner_size()
                    .expect("Dimensions are wrong.")
                    .to_physical(self.surface.window().get_hidpi_factor())
                    .into();
                [d.0, d.1]
            };
            let (new_swapchain, new_images) = self
                .swapchain
                .recreate_with_dimension(self.dimensions)
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
                self.device.clone(),
                self.depth_buffer.clone(),
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
    CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        e.model.positions.iter().copied(),
    )
    .expect("failed to create buffer");
}
fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
    _device: Arc<Device>,
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
