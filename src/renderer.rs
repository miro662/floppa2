use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::pipeline::Pipeline;

#[derive(Debug, PartialEq, Eq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    capabilities: wgpu::SurfaceCapabilities,

    surface: wgpu::Surface,
    window_size: WindowSize,

    pipeline: Pipeline,
}

impl Renderer {
    pub fn compatible_with<T>(window: T, window_size: WindowSize) -> Renderer
    where
        T: HasRawWindowHandle + HasRawDisplayHandle,
    {
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = pollster::block_on(Self::get_adapter(&instance, &surface));
        let (device, queue) = pollster::block_on(Self::get_compatible_device_queue(&adapter));
        let capabilities = surface.get_capabilities(&adapter);
        let pipeline = Pipeline::new(&device, capabilities.formats[0].into());

        let renderer = Renderer {
            device,
            queue,
            surface,
            window_size,
            capabilities,
            pipeline,
        };
        renderer.reconfigure_surface();
        renderer
    }

    pub fn resize(&mut self, new_size: WindowSize) {
        self.window_size = new_size;
        self.reconfigure_surface();
    }

    // TODO: redesign me
    pub fn render(&self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.queue
            .submit(Some(self.pipeline.render(&self.device, &view)));
        frame.present();
    }

    async fn get_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(surface),
            })
            .await
            .unwrap()
    }

    async fn get_compatible_device_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        let device_descriptor = wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        };
        adapter
            .request_device(&device_descriptor, None)
            .await
            .unwrap()
    }

    fn reconfigure_surface(&self) {
        let format = self.capabilities.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: self.window_size.width,
            height: self.window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: self.capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        self.surface.configure(&self.device, &config);
    }
}
