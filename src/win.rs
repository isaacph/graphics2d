use std::sync::Arc;

use pollster::FutureExt;
use wgpu::{DeviceDescriptor, InstanceDescriptor, PowerPreference, RequestAdapterOptions, SurfaceConfiguration, TextureUsages};
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ControlFlow, EventLoop}, window::Window};

pub trait Client {
    fn draw(&mut self, render_context: &mut RenderContext);
    fn resize(&mut self, render_context: &mut RenderContext, _size: winit::dpi::PhysicalSize<u32>);
}

pub struct App<C: Client, F: FnOnce(&mut RenderContext) -> C> {
    window: Option<Arc<Window>>,
    render_context: Option<RenderContext>,
    client: Option<C>,
    init_func: Option<F>,
}

pub struct RenderContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_format: wgpu::TextureFormat,
    pub surface: wgpu::Surface<'static>,
    pub window: Arc<Window>,
}

pub fn run<C: Client, F: FnOnce(&mut RenderContext) -> C>(init_func: F) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::<C, F> {
        window: None,
        render_context: None,
        client: None,
        init_func: Some(init_func),
    }).unwrap();
}

impl<C: Client, F: FnOnce(&mut RenderContext) -> C> ApplicationHandler for App<C, F> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attributes = Window::default_attributes()
            .with_title("Test");
        self.window = Some(Arc::new(event_loop.create_window(attributes).unwrap()));
        let window = self.window.as_mut().unwrap().clone();
        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface: wgpu::Surface<'static> = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }).block_on().unwrap();
        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            label: Some("Test device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }).block_on().unwrap();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);
        let size = window.inner_size();
        self.render_context = Some(RenderContext {
            instance,
            adapter,
            device,
            queue,
            surface_format,
            surface,
            window,
        });
        let init_func = self.init_func.take().unwrap();
        self.client = Some(init_func(self.render_context.as_mut().unwrap()));
        self.configure_window(size);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let _ = window_id;
        match self.window.as_mut() {
            Some(window) => window,
            None => return,
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.draw();
                self.window.as_mut().unwrap().request_redraw();
            },
            WindowEvent::Resized(size) => self.configure_window(size),
            _ => (),
        };
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }
}

impl<C: Client, F: FnOnce(&mut RenderContext) -> C> App<C, F> {
    fn configure_window(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        let render_state = self.render_context.as_mut().unwrap();
        render_state.surface.configure(&render_state.device, &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: render_state.surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![render_state.surface_format],
        });
        self.resize(size);
    }

    fn draw(&mut self) {
        self.client.as_mut().unwrap()
            .draw(self.render_context.as_mut().unwrap());
    }
    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.client.as_mut().unwrap()
            .resize(self.render_context.as_mut().unwrap(), size);
    }
}
