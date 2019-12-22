use crate::Renderer;
use std::sync::Arc;
use vulkano::device::{Device, Queue};

impl Renderer {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> Self {
        Self {}
    }
}
