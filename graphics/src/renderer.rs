use crate::{
    shaders::{cs, fs, vs},
    Normal,
    Renderer,
    Ubo,
    Vertex,
};
use common::{managers::AssetManager, Quaternion};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        AutoCommandBufferBuilder,
        CommandBuffer,
        DynamicState,
        SubpassContents,
    },
    descriptor::{
        descriptor_set::PersistentDescriptorSet,
        pipeline_layout::PipelineLayoutAbstract,
    },
    device::{Device, Queue},
    framebuffer::{FramebufferAbstract, RenderPassAbstract, Subpass},
    image::SwapchainImage,
    pipeline::{vertex::TwoBuffersDefinition, ComputePipeline, GraphicsPipeline},
    swapchain::{self, AcquireError, Swapchain},
    sync,
    sync::GpuFuture,
};

use winit::window::Window;

impl Renderer {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        images: &[Arc<SwapchainImage<Window>>],
        render_pass: Arc<dyn RenderPassAbstract + Sync + Send>,
    ) -> Self {
        let cs = cs::Shader::load(device.clone()).unwrap();
        let vs = vs::Shader::load(device.clone()).unwrap();
        let fs = fs::Shader::load(device.clone()).unwrap();
        let data_buffer: Arc<dyn BufferAccess + Send + Sync> = {
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                true,
                AssetManager::new()
                    .load::<f32>("sphere")
                    .set_scale(0.05)
                    .model
                    .positions
                    .iter()
                    .cloned(),
            )
            .unwrap()
        };
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                // .vertex_input_single_buffer::<Vertex>()
                .vertex_input(TwoBuffersDefinition::<Vertex,Normal>::new())
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .depth_stencil_simple_depth()
                .line_width(1.0)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );
        let compute_pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &cs.main_entry_point(),
                &(),
                None,
            )
            .expect("failed to create compute pipeline"),
        );
        let compute_layout =
            compute_pipeline.layout().descriptor_set_layout(0).unwrap();
        let previous_frame_end = Some(Self::create_sync_objects(&device));

        Self {
            pipeline,

            compute_pipeline: compute_pipeline.clone(),
            // compute_command_buffer,
            previous_frame_end,
            data_buffer,
            compute_layout: compute_layout.clone(),
        }
    }

    pub fn compute(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        dimensions: [u32; 2],
        input: &AssetManager,
        mouse: [f64; 2],
        images: &[Arc<SwapchainImage<Window>>],
        //-> Arc<dyn BufferAccess + Send + Sync>
    ) {
        let uniform_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            true,
            vec![Ubo {
                ar:    [
                    images[0].dimensions()[0] as f32 / 2560.0,
                    images[0].dimensions()[1] as f32 / 1440.0,
                ],
                mouse: [
                    mouse[0] as f32 / dimensions[0] as f32 * 2.0 - 1.0,
                    -(mouse[1] as f32 / dimensions[1] as f32 * 2.0 - 1.0),
                ],
                proj:  [[0.0f32; 4]; 4],
                rot:   Quaternion::new([
                    (mouse[1] as f32 / dimensions[1] as f32).cos(),
                    -(mouse[0] as f32 / dimensions[0] as f32).sin(),
                    -(mouse[0] as f32 / dimensions[0] as f32).sin(),
                    (mouse[1] as f32 / dimensions[1] as f32).sin(),
                ])
                .u_mut()
                .val,
            }]
            .iter()
            .cloned(),
        )
        .expect("Failed to create data_buffer");
        let data_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            true,
            {
                let mut e = input.load::<f32>("sphere");
                let o = Quaternion::new([1.0, 0.0, 0.0, 0.0]).u_mut().val;
                e.model.scale = 0.05;
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
                        orient:   o,
                        normals:  [0.0f32; 4],
                    });
                }
                x
            }
            .iter()
            .cloned(),
        )
        .unwrap();

        let set = Arc::new(
            PersistentDescriptorSet::start(self.compute_layout.clone())
                .add_buffer(data_buffer.clone())
                .unwrap()
                .add_buffer(uniform_buffer)
                .unwrap()
                .build()
                .unwrap(),
        );
        let mut compute_builder =
            AutoCommandBufferBuilder::new(device.clone(), queue.family())
                .unwrap();
        compute_builder
            .dispatch(
                [1024, 1, 1],
                self.compute_pipeline.clone(),
                set.clone(),
                (),
            )
            .unwrap();
        let compute_command_buffer = Arc::new(compute_builder.build().unwrap());

        let finished = compute_command_buffer.execute(queue).unwrap();
        finished
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
        self.data_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::vertex_buffer(),
            true,
            data_buffer.read().unwrap().iter().cloned(),
        )
        .unwrap();
        // let content = data_buffer.read().unwrap();
        // for (n, val) in content.iter().enumerate() {
        //     println!("{} {:?}", n, val);
        // }
    }

    fn create_sync_objects(
        device: &Arc<Device>
    ) -> Box<dyn GpuFuture + Send + Sync> {
        Box::new(sync::now(device.clone())) as Box<dyn GpuFuture + Send + Sync>
    }

    pub fn render(
        &mut self,
        queue: Arc<Queue>,
        device: Arc<Device>,
        swapchain: Arc<Swapchain<Window>>,
        framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
        dynamic_state: &mut DynamicState,
    ) -> bool {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        // let vertex_buffer: Arc<dyn BufferAccess + Send + Sync> = {
        //     CpuAccessibleBuffer::from_iter(
        //         device.clone(),
        //         BufferUsage::vertex_buffer(),
        //         AssetManager::new()
        //             .load::<f32>("teapot")
        //             .set_scale(0.05)
        //             .model
        //             .positions
        //             .iter()
        //             .cloned(),
        //     )
        //     .unwrap()
        // };
        let (index_buffer, normals_buffer) = {
            let e = AssetManager::new().load::<f32>("sphere").set_scale(0.05);
            (
                CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::index_buffer(),
                    true,
                    e.model.indices.iter().cloned(),
                )
                .expect("Failed to create index_buffer"),
                CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::all(),
                    true,
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
        let clear_values = vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()];
        let (image_num, _, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    return true;
                }
                Err(err) => panic!("{:?}", err),
            };
        let mut command_buffer_builder =
            AutoCommandBufferBuilder::primary(device.clone(), queue.family())
                .unwrap();
        command_buffer_builder
            .begin_render_pass(
                framebuffers[image_num].clone(),
                SubpassContents::Inline,
                clear_values,
            )
            .unwrap()
            .draw_indexed(
                self.pipeline.clone(),
                &dynamic_state,
                vec![self.data_buffer.clone(), normals_buffer],
                index_buffer,
                (),
                (),
            )
            .unwrap()
            .end_render_pass()
            .unwrap();
        let command_buffer = Arc::new(command_buffer_builder.build().unwrap());

        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(queue, swapchain, image_num)
            .then_signal_fence_and_flush();
        match future {
            Ok(future) => {
                // This wait is required when using NVIDIA or running on macOS.
                // See https://github.com/vulkano-rs/vulkano/issues/1247
                // future.wait(None).unwrap(); // winit <=0.19
                self.previous_frame_end = Some(Box::new(future) as Box<_>);
                false
            }
            Err(sync::FlushError::OutOfDate) => {
                self.previous_frame_end =
                    Some(Box::new(vulkano::sync::now(device)) as Box<_>);
                true
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end =
                    Some(Box::new(vulkano::sync::now(device)) as Box<_>);
                false
                // true
            }
        }

        // let content = data_buffer.read().unwrap();
        // for (n, val) in content.iter().enumerate() {
        //     println!("{} {:?}", n, val);
        // }
    }
}
