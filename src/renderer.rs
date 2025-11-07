use std::sync::Arc;
use winit::window::Window;


pub struct Renderer {
	window: Arc<Window>,
}

impl Renderer {
	pub async fn new(window: Window) -> anyhow::Result<Self> {
		Ok(Self {
			window: Arc::new(window),
		})
	}

    pub fn resize(&mut self, width: u32, height: u32) {
    }
    
    pub fn render(&mut self) {
    }
}