use engine::Engine;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() {
    let event_loop = EventLoop::new();
    let mut engine = Engine::<f32>::new(&event_loop);
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            engine.resize_window();
        }
        Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } => {
            if [position.x, position.y] != [0.0f64; 2] {
                engine.setmouse([position.x, position.y])
            }
        }
        Event::RedrawEventsCleared => {
            //TODO: HiDPI scaling as push constants
            engine.run();
        }
        _ => (),
    });
}
