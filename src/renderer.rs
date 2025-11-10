use std::sync::Arc;
use winit::window::Window;

use anyhow::{Context, anyhow};

pub struct Renderer {
	device: wgpu::Device,
	queue: wgpu::Queue,
	surface: wgpu::Surface<'static>,
	surface_config: wgpu::SurfaceConfiguration,
	window: Arc<Window>,
}

impl Renderer {

	//public

	pub async fn new(window: Window) -> anyhow::Result<Self> {

		let size = window.inner_size();

		let window = Arc::new(window);

		let instance = Self::create_instance();

		let surface = Self::create_surface(&instance, window.clone())?;

		let adapter = Self::request_adapter(&instance, &surface).await?;

		let (device, queue) = Self::request_device(&adapter).await?;

		let surface_caps = surface.get_capabilities(&adapter);
		let surface_format = Self::find_surface_format(&surface_caps)?;
		let alpha_mode = Self::find_alpha_mode(&surface_caps)?;

		let surface_config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::AutoVsync,
			desired_maximum_frame_latency: 2,
			alpha_mode: alpha_mode,
			view_formats: vec![surface_format.add_srgb_suffix()],
		};

		surface.configure(&device, &surface_config);

		Ok(Self {
			device,
			queue,
			surface,
			surface_config,
			window,
		})
	}

    pub fn resize(&mut self, width: u32, height: u32) {
		if 0 < width && 0 < height {
			self.surface_config.width = width;
			self.surface_config.height = height;
			self.surface.configure(&self.device, &self.surface_config);
		}
    }
    
    pub fn render(&mut self) {

		let frame = match self.surface.get_current_texture() {
			Ok(frame) => frame,
			Err(wgpu::SurfaceError::Outdated) | Err(wgpu::SurfaceError::Lost) => {
				let size = self.window.inner_size();
				self.resize(size.width, size.height);
				return;
			},
			Err(e) => {
				log::error!("Failed to acquire next swap chain texture: {:?}", e);
				return;
			},
		};

		let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
			label: Some("Render Texture View"),
			format: Some(self.surface_config.format.add_srgb_suffix()),
			..Default::default()
		});

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Command Encoder"),
		});

		let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Render Pass"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &view,
				depth_slice: None,
				resolve_target: None,
				ops: wgpu::Operations {
					//TEAL
					load: wgpu::LoadOp::Clear(wgpu::Color {
						r: 0.016,
						g: 0.545,
						b: 0.604,
						a: 1.0,
					}),
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
		});

		drop(render_pass);

		self.queue.submit(std::iter::once(encoder.finish()));
		self.window.pre_present_notify();
		frame.present();
    }

	//private

	fn create_instance() -> wgpu::Instance {
		wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default())
	}

	fn create_surface(instance: &wgpu::Instance, window: Arc<Window>) -> anyhow::Result<wgpu::Surface<'static>> {
		instance.create_surface(window).context("Failed to create wgpu surface")
	}

	async fn request_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface<'_>) -> anyhow::Result<wgpu::Adapter> {
		instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::from_env().unwrap_or(wgpu::PowerPreference::HighPerformance),
				force_fallback_adapter: false,
				compatible_surface: Some(surface),
			},
		).await.context("Failed to request wgpu adapter")
	}

	async fn request_device(adapter: &wgpu::Adapter) -> anyhow::Result<(wgpu::Device, wgpu::Queue)> {
		adapter.request_device(
			&wgpu::DeviceDescriptor {
				label: Some("Renderer Device"),
				required_features: wgpu::Features::empty(),
				required_limits: wgpu::Limits::default(),
				experimental_features: wgpu::ExperimentalFeatures::disabled(),
				memory_hints: wgpu::MemoryHints::Performance,
				trace: wgpu::Trace::Off,
			},
		).await.context("Failed to request wgpu device")
	}

	fn find_surface_format(surface_caps: &wgpu::SurfaceCapabilities) -> anyhow::Result<wgpu::TextureFormat> {
		surface_caps.formats.first().copied().ok_or(anyhow!("No supported surface formats found (surface is incompatible with adapter)"))
	}

	fn find_alpha_mode(surface_caps: &wgpu::SurfaceCapabilities) -> anyhow::Result<wgpu::CompositeAlphaMode> {
		let alpha_mode_preference = |mode: wgpu::CompositeAlphaMode| {
			match mode {
				wgpu::CompositeAlphaMode::Inherit => 1,
				wgpu::CompositeAlphaMode::PreMultiplied => 2,
				wgpu::CompositeAlphaMode::PostMultiplied => 3,
				wgpu::CompositeAlphaMode::Opaque => 4,
				_ => 5,
			}
		}; 

		surface_caps.alpha_modes.iter().min_by_key(|mode| alpha_mode_preference(**mode)).copied().ok_or(anyhow!("No supported alpha modes found (normaly should not happen)"))
	}

}