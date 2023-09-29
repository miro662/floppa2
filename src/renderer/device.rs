use cgmath::Vector2;
use raw_window_handle::HasRawDisplayHandle;
use raw_window_handle::HasRawWindowHandle;

pub(crate) struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_format: wgpu::TextureFormat,
    alpha_mode: wgpu::CompositeAlphaMode,
}

impl Gpu {
    pub(crate) fn compatible_with<T>(window: T, size: impl Into<Vector2<u32>>) -> Gpu
    where
        T: HasRawWindowHandle + HasRawDisplayHandle,
    {
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = pollster::block_on(Self::get_adapter(&instance, &surface));
        let (device, queue) = pollster::block_on(Self::get_compatible_device_queue(&adapter));

        let capabilities = surface.get_capabilities(&adapter);
        let surface_format = capabilities.formats[0];
        let alpha_mode = capabilities.alpha_modes[0];

        let gpu = Gpu {
            device,
            queue,
            surface,
            surface_format,
            alpha_mode,
        };
        gpu.resize(size);
        gpu
    }

    pub(crate) fn resize(&self, size: impl Into<Vector2<u32>>) {
        let size = size.into();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            alpha_mode: self.alpha_mode,
            width: size.x,
            height: size.y,
            present_mode: wgpu::PresentMode::Fifo,
            view_formats: vec![],
        };
        self.surface.configure(&self.device, &config);
    }

    pub(crate) fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub(crate) fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub(crate) fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub(crate) fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
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
}
