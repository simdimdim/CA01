use graphics::Graphics;
use winit::{Event, EventsLoop, WindowEvent};
pub struct Engine {
    events_loop: EventsLoop,
    graphics:    Graphics,
}

impl Default for Engine {
    fn default() -> Self { Self::new() }
}

impl Engine {
    pub fn new() -> Self {
        let events_loop = EventsLoop::new();
        let graphics = Graphics::new(&events_loop);
        Self {
            events_loop,
            graphics,
        }
    }

    pub fn run(mut self) {
        let mut mouse: (f64, f64) = (0.0, 0.0);
        loop {
            let mut done = false;
            let mut recreate_swapchain = false;
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
                } => mouse = (position.x, position.y),
                _ => (),
            });
            self.graphics.render(recreate_swapchain, mouse);
            if done {
                return;
            }
        }
    }
}
