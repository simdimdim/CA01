pub mod assetmanager;
pub mod graphics;

use std::{path::PathBuf, sync::Arc};
use tobj::Model;
use vulkano::{
    command_buffer::DynamicState,
    device::{Device, Queue},
    framebuffer::{FramebufferAbstract, RenderPassAbstract},
    image::attachment::AttachmentImage,
    instance::Instance,
    swapchain::{Surface, Swapchain},
    sync::GpuFuture,
};
use winit::Window;

#[derive(Default)]
pub struct AssetManager {
    assets_path: PathBuf,
    names:       Vec<String>,
    objects:     Vec<Model>,
}
pub struct Graphics {
    instance:           Arc<Instance>,
    surface:            Arc<Surface<Window>>,
    device:             Arc<Device>,
    queue:              Arc<Queue>,
    render_pass:        Arc<dyn RenderPassAbstract + Sync + Send>,
    framebuffers:       Vec<Arc<(dyn FramebufferAbstract + Send + Sync)>>,
    swapchain:          Arc<Swapchain<Window>>,
    previous_frame_end: Option<Box<dyn GpuFuture + Send + Sync>>,
    recreate_swapchain: bool,
    dimensions:         [u32; 2],
    images:             Vec<Arc<vulkano::image::SwapchainImage<winit::Window>>>,
    dynamic_state:      DynamicState,
    /* pipeline:           Arc<dyn GraphicsPipelineAbstract + Sync + Send>,
     * vertex_buffer:      Vec<Arc<dyn BufferAccess + Send + Sync>>,
     * acquire_future:     SwapchainAcquireFuture<Window>,
     * command_buffer:     Arc<AutoCommandBuffer>, */
    depth_buffer:       Arc<AttachmentImage>,
}
