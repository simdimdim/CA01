use engine::Engine;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
fn main() {
    let event_loop = EventLoop::new();
    let mut engine = Engine::<f32>::new(&event_loop);
    let mut recreate_swapchain = false;
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
            recreate_swapchain = true;
        }
        Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } => {
            if [position.x, position.y] != [0; 2] {
                engine.setmouse([position.x.into(), position.y.into()])
            }
        }
        Event::RedrawEventsCleared => {
            //TODO: HiDPI scaling as push constants
            engine.run(recreate_swapchain);
        }
        _ => (),
    });
}
