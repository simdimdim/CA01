pub mod world;
use common::{managers::AssetManager, Entity, Quaternion};
use graphics::Graphics;
use num_traits::Float;
use winit::{Event, EventsLoop, WindowEvent};

pub struct Engine<T: Float + From<f32>> {
    events_loop: EventsLoop,
    graphics:    Graphics,
    mouse:       [f64; 2],
    world:       World<T>,
}

#[derive(Clone, Debug)]
pub struct World<T: Float + From<f32>> {
    origin:  Quaternion<T>,
    objects: Vec<Entity<T>>,
}

impl<T: std::fmt::Debug + Float + From<f32>> Engine<T> {
    pub fn new() -> Self {
        let events_loop = EventsLoop::new();
        let graphics = Graphics::new(&events_loop);
        let mouse = [0.0f64; 2];
        let world = Self::create_world();
        Self {
            events_loop,
            graphics,
            mouse,
            world,
        }
    }

    pub fn run(mut self) {
        loop {
            let mut done = false;
            let mut recreate_swapchain = false;
            //TODO: HiDPI scaling as push constants
            let mut mouse = [0.0f64; 2];
            self.events_loop.poll_events(|ev| match ev {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => done = true,
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => recreate_swapchain = true,
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => mouse = [position.x, position.y],
                _ => (),
            });
            if mouse != [0.0f64, 0.0f64] {
                self.mouse = mouse;
            }
            self.graphics.render(recreate_swapchain, self.mouse);
            if done {
                return;
            }
        }
    }

    pub fn create_world() -> World<T> {
        let world = World::<T>::new();
        world.add_object(AssetManager::new().load::<T>("cube"));
        world
    }
}
