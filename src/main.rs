use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use anyhow::Context;

struct App {
    window: Option<Window>,
    result: anyhow::Result<()>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            result: Ok(()),
        }
    }
}

impl App {
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        let window_attributes = Window::default_attributes().with_title("RT");
        let window = event_loop.create_window(window_attributes).context("Failed to create window")?;
        self.window = Some(window);
        log::info!("Window created");
        Ok(())
    }

}

impl ApplicationHandler for App {
    /*
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(event_loop.create_window(Window::default_attributes()).unwrap());
        log::info!("Window created");
    }
    */

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.result = match cause {
            StartCause::Init => self.create_window(event_loop),
            _ => Ok(()),
        };
        if self.result.is_err() {
            event_loop.exit();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                log::info!("Redraw Request");
                //self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new().context("Failed to create event loop")?;

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).context("Application run failed")?;
    app.result
}