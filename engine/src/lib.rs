pub mod world;
use common::{managers::AssetManager, Entity, Quaternion};
use graphics::Graphics;
use num_traits::Float;
use winit::{Event, EventsLoop, WindowEvent};

#[allow(dead_code)]
pub struct Engine<T: Float + From<f32>> {
    events_loop: EventsLoop,
    graphics:    Graphics,
    mouse:       [f64; 2],
    world:       World<T>,
    assets:      AssetManager,
}

#[derive(Clone, Debug)]
pub struct World<T: Float + From<f32>> {
    origin:  Quaternion<T>,
    objects: Vec<Entity<T>>,
}

impl<T: std::fmt::Debug + Float + From<f32>> Default for Engine<T> {
    fn default() -> Self { Self::new() }
}

impl<T: std::fmt::Debug + Float + From<f32>> Engine<T> {
    pub fn new() -> Self {
        let events_loop = EventsLoop::new();
        let graphics = Graphics::new(&events_loop);
        let mouse = [0.0f64; 2];
        let world = Self::create_world();
        let assets = AssetManager::new();
        Self {
            events_loop,
            graphics,
            mouse,
            world,
            assets,
        }
    }

    pub fn run(mut self) {
        let mut mouse: [f64; 2] = [0.0; 2];
        loop {
            let mut done = false;
            let mut recreate_swapchain = false;
            //TODO: HiDPI scaling as push constants
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
                } => {
                    if [position.x, position.y] != [0.0; 2] {
                        mouse = [position.x, position.y]
                    }
                }
                _ => (),
            });
            self.mouse = mouse;
            self.graphics
                .render(recreate_swapchain, &self.assets, self.mouse);
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
