pub mod world;

use common::{managers::AssetManager, Entity, Quaternion};
use graphics::Graphics;
use num_traits::Float;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
pub struct Engine<T: Float + From<f32>> {
    graphics: Graphics,
    mouse:    [f64; 2],
    world:    World<T>,
    assets:   AssetManager,
}

#[derive(Clone, Debug)]
pub struct World<T: Float + From<f32>> {
    origin:  Quaternion<T>,
    objects: Vec<Entity<T>>,
}

impl<T: std::fmt::Debug + Float + From<f32>> Engine<T> {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mouse = [0.0f64; 2];
        let world = Self::create_world();
        let assets = AssetManager::new();
        let graphics = Graphics::new(event_loop);
        Self {
            graphics,
            mouse,
            world,
            assets,
        }
    }

    pub fn setmouse(
        &mut self,
        mouss: [f64; 2],
    ) {
        self.mouse = mouss;
    }

    pub fn run(
        &mut self,
        recreate_swapchain: bool,
    ) {
        // self.graphics.surface
        // &event_loop.run(move |event, _, control_flow| match event {
        //     Event::WindowEvent {
        //         event: WindowEvent::CloseRequested,
        //         ..
        //     } => {
        //         done = true;
        //         *control_flow = ControlFlow::Exit;
        //     }
        //     Event::WindowEvent {
        //         event: WindowEvent::Resized(_),
        //         ..
        //     } => {
        //         recreate_swapchain = true;
        //     }
        //     Event::WindowEvent {
        //         event: WindowEvent::CursorMoved { position, .. },
        //         ..
        //     } => {
        //         if [position.x, position.y] != [0; 2] {
        //             self.mouse = [position.x.into(), position.y.into()]
        //         }
        //     }
        //     Event::RedrawEventsCleared => {
        //         //TODO: HiDPI scaling as push constants
        //         self.graphics.render(
        //             recreate_swapchain,
        //             &self.assets,
        //             self.mouse,
        //         );
        //     }
        //     _ => (),
        // });
        self.graphics
            .render(recreate_swapchain, &self.assets, self.mouse);
    }

    pub fn create_world() -> World<T> {
        let world = World::<T>::new();
        world.add_object(AssetManager::new().load::<T>("cube"));
        world
    }
}
