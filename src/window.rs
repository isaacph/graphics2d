use wgpu::InstanceFlags;
use winit::event_loop::EventLoop;
use std::collections::HashSet;
use std::rc::Rc;

use winit::{
    event::*,
    event_loop::ControlFlow,
    window::WindowBuilder,
};
use winit::window::Window;

pub fn run<F>(create_state: F)
        where F: FnOnce(&mut Resources) -> Box<dyn StateTrait> {
    pollster::block_on(run_async(create_state));
}

pub async fn run_async<F>(create_state: F)
        where F: FnOnce(&mut Resources) -> Box<dyn StateTrait> {
    // make webassembly work
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let window = Rc::new(WindowBuilder::new().build(&event_loop).unwrap());

    #[cfg(target_arch = "wasm32")]
    let c_window = window.clone();
    #[cfg(target_arch = "wasm32")]
    let resize_closure = wasm_bindgen::prelude::Closure::<dyn FnMut()>::new(move || {
        web_sys::window()
            .and_then(|win| Some((win.document().unwrap(), win.device_pixel_ratio())))
            .and_then(|(doc, ratio)| {
                use winit::dpi::PhysicalSize;
                let dst = doc.get_element_by_id("wasm-example")?;
                let width = (dst.client_width() as f64 * ratio) as i32;
                let height = (dst.client_height() as f64 * ratio) as i32;
                c_window.set_inner_size(PhysicalSize::new(width, height));
                Some(())
            })
            .expect("Couldn't resize");
    });


    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast;
        // window.set_inner_size(PhysicalSize::new(800, 600));
        
        use winit::platform::web::WindowExtWebSys;
        let window = window.clone();
        web_sys::window()
            .and_then(|win| {
                let window = window.clone();
                win.set_onresize(Some(resize_closure.as_ref().unchecked_ref()));
                Some((win.document().unwrap(), win.device_pixel_ratio()))
            })
            .and_then(|(doc, ratio)| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let width = (dst.client_width() as f64 * ratio) as i32;
                let height = (dst.client_height() as f64 * ratio) as i32;
                window.set_inner_size(PhysicalSize::new(width, height));
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut context = Context::new(&window, create_state).await;

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => {
                    context.close(&window);
                    elwt.exit();
                },
                WindowEvent::Resized(physical_size) => {
                    context.resize(&window, *physical_size);
                }
                WindowEvent::RedrawRequested => {
                    match context.render(&window) {
                        Ok(_) => {},
                        // Reconfigure the surface if lsot
                        Err(wgpu::SurfaceError::Lost) => context.resize(&window, context.resources.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                },
                WindowEvent::ModifiersChanged(modifiers) => {
                    context.update_modifiers(*modifiers);
                },
                _ => {
                    context.input(&window, event);
                },
            },
            Event::AboutToWait => {
                match context.update(&window) {
                    true => elwt.exit(),
                    false => (),
                };
                window.request_redraw();
            },
            _ => {}
        }
    });
}

pub struct Resources {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub modifiers: Modifiers,
}

pub trait StateTrait {
    fn resize(&mut self, window: &Window, resources: &mut Resources, new_size: winit::dpi::PhysicalSize<u32>);
    fn input(&mut self, window: &Window, resources: &mut Resources, event: &WindowEvent) -> bool;
    fn update(&mut self, window: &Window, resources: &mut Resources) -> bool;
    fn render(&mut self, window: &Window, resources: &mut Resources) -> Result<(), wgpu::SurfaceError>;
    fn close(&mut self, window: &Window, resources: &mut Resources);
}

pub struct Context {
    state: Box<dyn StateTrait>,
    resources: Resources,
}

impl Context {
    // Creating some of the wgpu types requires async code
    async fn new<F>(window: &Window, create_state: F) -> Self
            where F: FnOnce(&mut Resources) -> Box<dyn StateTrait> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: InstanceFlags::DEBUG.union(InstanceFlags::VALIDATION),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        };
        let instance = wgpu::Instance::new(instance_descriptor);
        // had to enable rwh_05 feature of winit 0.29.8 to get the right trait on window
        // for the current version of wgpu
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let present_mode = if surface_caps.present_modes.iter()
            .any(|mode| *mode == wgpu::PresentMode::Fifo) {
            wgpu::PresentMode:: Fifo
        } else { surface_caps.present_modes[0] };
        // Shader code in the tutorial (that this came from) assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let mut resources = Resources {
            surface,
            device,
            queue,
            config,
            size,
            modifiers: Default::default(),
        };
        let state = create_state(&mut resources);

        Self {
            resources,
            state,
        }
    }
    fn update_modifiers(&mut self, new_modifiers: Modifiers) {
        self.resources.modifiers = new_modifiers;
    }
    fn resize(&mut self, window: &Window, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.resources.size = new_size;
            self.resources.config.width = new_size.width;
            self.resources.config.height = new_size.height;
            self.resources.surface.configure(&self.resources.device, &self.resources.config);
            self.state.resize(window, &mut self.resources, new_size)
        }
    }
    fn input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state.input(window, &mut self.resources, event)
    }
    fn update(&mut self, window: &Window) -> bool {
        self.state.update(window, &mut self.resources)
    }
    fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        self.state.render(window, &mut self.resources)
    }
    fn close(&mut self, window: &Window) {
        self.state.close(window, &mut self.resources)
    }
}
