use std::sync::Arc;
use winit::window::Window;


pub struct Renderer {
	window: Arc<Window>,
	surface: wgpu::Surface<'static>,
}

impl Renderer {
	pub async fn new(window: Window) -> anyhow::Result<Self> {

		let window = Arc::new(window);

		let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

		let surface = instance.create_surface(window.clone())?;

		Ok(Self {
			window,
			surface,
		})
	}

    pub fn resize(&mut self, width: u32, height: u32) {
    }
    
    pub fn render(&mut self) {
    }
}