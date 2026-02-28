use std::sync::Arc;
use winit::window::Window;
use wesl::include_wesl;

use anyhow::{Context, anyhow};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::NoUninit)]
struct Immediate {
	window_size: [u32; 2],
	aspect_ratio: [f32; 2],
}

impl Immediate {
	pub fn new(window_width: u32, window_height: u32) -> Self {
		Self {
			window_size: [window_width, window_height],
			aspect_ratio: Self::compute_aspect_ratio(window_width, window_height),
		}
	}

	pub fn update_window_size(&mut self, window_width: u32, window_height: u32) {
		self.window_size = [window_width, window_height];
		self.aspect_ratio = Self::compute_aspect_ratio(window_width, window_height);
	}

	pub fn compute_aspect_ratio(window_width: u32, window_height: u32) -> [f32; 2] {
		if window_width < window_height {
			[1.0, window_height as f32 / window_width as f32]
		} else {
			[window_width as f32 / window_height as f32, 1.0]
		}
	}
}

pub struct Renderer {
	render_pipeline: wgpu::RenderPipeline,
	device: wgpu::Device,
	queue: wgpu::Queue,
	surface: wgpu::Surface<'static>,
	surface_config: wgpu::SurfaceConfiguration,
	immediate: Immediate,
	window: Arc<Window>,
}

macro_rules! load_shader {
    ($device:expr, $path:literal, $label:literal) => {
        $device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some($label),
            source: wgpu::ShaderSource::Wgsl(include_wesl!($path).into()),
        })
    };
}

impl Renderer {

	//public

	pub async fn new(window: Window) -> anyhow::Result<Self> {
		let size = window.inner_size();

		let immediate = Immediate::new(size.width, size.height);

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

		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[],
			immediate_size: size_of::<Immediate>().try_into()?,
		});

		let render_pipeline = Self::create_render_pipeline(&device, render_pipeline_layout, &surface_config);

		Ok(Self {
			render_pipeline,
			device,
			queue,
			surface,
			surface_config,
			immediate,
			window,
		})
	}

    pub fn resize(&mut self) {
		let size = self.window.inner_size();
		if 0 < size.width && 0 < size.height {
			self.surface_config.width = size.width;
			self.surface_config.height = size.height;
			self.surface.configure(&self.device, &self.surface_config);
			self.immediate.update_window_size(size.width, size.height);
		}
    }
    
    pub fn render(&mut self) {
		let frame = match self.surface.get_current_texture() {
			Ok(frame) => frame,
			Err(wgpu::SurfaceError::Outdated) | Err(wgpu::SurfaceError::Lost) => {
				self.resize();
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

		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Render Pass"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &view,
				depth_slice: None,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
			multiview_mask: None,
		});

		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_immediates(0, bytemuck::bytes_of(&self.immediate));
		render_pass.draw(0..3, 0..1);

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
				required_features: wgpu::Features::IMMEDIATES,
				required_limits: wgpu::Limits {
					max_immediate_size: size_of::<Immediate>().try_into()?,
					..wgpu::Limits::default()
				},
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

	fn create_render_pipeline(device: &wgpu::Device, render_pipeline_layout: wgpu::PipelineLayout, surface_config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
		device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Main Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &load_shader!(device, "vertex_shader", "Vertex Shader"),
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &load_shader!(device, "fragment_shader", "Fragment Shader"),
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
	}

}