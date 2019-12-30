pub mod graphics;
pub mod renderer;
pub mod shaders;

use std::sync::Arc;
use vulkano::{
    buffer::{BufferAccess, CpuAccessibleBuffer},
    command_buffer::DynamicState,
    descriptor::descriptor_set::UnsafeDescriptorSetLayout,
    device::{Device, Queue},
    framebuffer::{FramebufferAbstract, RenderPassAbstract},
    image::attachment::AttachmentImage,
    pipeline::{ComputePipelineAbstract, GraphicsPipelineAbstract},
    swapchain::{Surface, Swapchain},
    sync::GpuFuture,
};
use winit::Window;

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
vulkano::impl_vertex!(Vertex, position, orient, normals);
vulkano::impl_vertex!(Normal, normal);

pub struct Renderer {
    pipeline:           Arc<dyn GraphicsPipelineAbstract + Sync + Send>,
    compute_pipeline:   Arc<dyn ComputePipelineAbstract + Sync + Send>,
    // compute_command_buffer: Arc<AutoCommandBuffer>,
    previous_frame_end: Option<Box<dyn GpuFuture + Send + Sync>>,
    data_buffer:        Arc<dyn BufferAccess + Send + Sync>,
    compute_layout:     Arc<UnsafeDescriptorSetLayout>,
}
pub struct Graphics {
    // instance:           Arc<Instance>,
    surface:            Arc<Surface<Window>>,
    device:             Arc<Device>,
    queue:              Arc<Queue>,
    render_pass:        Arc<dyn RenderPassAbstract + Sync + Send>,
    framebuffers:       Vec<Arc<(dyn FramebufferAbstract + Send + Sync)>>,
    swapchain:          Arc<Swapchain<Window>>,
    recreate_swapchain: bool,
    dimensions:         [u32; 2],
    images:             Vec<Arc<vulkano::image::SwapchainImage<winit::Window>>>,
    dynamic_state:      DynamicState,
    depth_buffer:       Arc<AttachmentImage>,
    renderer:           Renderer,
}
