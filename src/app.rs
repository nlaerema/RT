use crate::renderer::Renderer;

use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{EventLoop, ActiveEventLoop};
use winit::window::{Window, WindowId};

use anyhow::Context;

pub struct App {
    renderer: Option<Renderer>,
    result: anyhow::Result<()>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            renderer: None,
            result: Ok(()),
        }
    }
}

impl App
where
    Self: ApplicationHandler,
{
    //public

   pub fn run(mut self) -> anyhow::Result<()> {
        let event_loop = EventLoop::new().context("Failed to create event loop")?;

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        event_loop.run_app(&mut self).context("Application run failed")?;
        self.result
    }

    //private

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<Window> {
        let window_attributes = Window::default_attributes().with_title("RT");
        let window = event_loop.create_window(window_attributes).context("Failed to create window")?;
        log::info!("Window created");
        Ok(window)
    }

    fn init_renderer(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        let window = self.create_window(event_loop)?;
        let renderer = pollster::block_on(Renderer::new(window))?;
        self.renderer = Some(renderer);
        log::info!("Renderer initialized");
        Ok(())
    }
}

impl ApplicationHandler for App {

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.result = match cause {
            StartCause::Init => self.init_renderer(event_loop),
            _ => Ok(()),
        };
        if self.result.is_err() {
            event_loop.exit();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                log::info!("Redraw Request");

                let renderer = self.renderer.as_mut().unwrap();

                renderer.render();
            },
            WindowEvent::CloseRequested => {
                log::info!("Close Requested");
                event_loop.exit();
            }
            _ => (),
        }
    }
}