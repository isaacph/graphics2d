use std::sync::Arc;
use winit::dpi::PhysicalSize;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

pub trait AppState {
    fn init(&mut self);
    fn draw(&mut self);
    fn exit(&mut self);
    fn event(&mut self, event: WindowEvent);
    fn resize(&mut self, size: cgmath::Vector2<u32>);
}

pub struct Window<'a, T: AppState> {
    pub app: T,
    render_state: Option<RenderState<'a>>,
    window: Option<Arc<winit::window::Window>>,
}

struct RenderState<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl<'a, T: AppState + 'static> ApplicationHandler for Window<'a, T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = winit::window::Window::default_attributes().with_title("Test window");
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

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
                        c_window.request_inner_size(PhysicalSize::new(width, height));
                        Some(())
                    })
                    .expect("Couldn't resize");
            });

            #[cfg(target_arch = "wasm32")]
            {
                // Winit prevents sizing with CSS, so we have to set
                // the size manually when on web.
                use wasm_bindgen::prelude::Closure;
                use wasm_bindgen::JsCast;
                // let _ = window.request_inner_size(PhysicalSize::new(450, 400));
                
                use winit::platform::web::WindowExtWebSys;
                let window = window.clone();
                web_sys::window()
                    .and_then(|win| {
                        win.set_onresize(Some(resize_closure.as_ref().unchecked_ref()));
                        Some((win.document().unwrap(), win.device_pixel_ratio()))
                    })
                    .and_then(|(doc, ratio)| {
                        let dst = doc.get_element_by_id("wasm-example")?;

                        let width = (dst.client_width() as f64 * ratio) as i32;
                        let height = (dst.client_height() as f64 * ratio) as i32;
                        window.request_inner_size(PhysicalSize::new(width, height));

                        let canvas = web_sys::Element::from(window.canvas()?);
                        dst.append_child(&canvas).ok()?;
                        Some(())
                    })
                    .expect("Couldn't append canvas to document body.");
            }

            let mut size = window.inner_size();
            if size.width == 0 || size.height == 0 {
                println!("Bad window size: {}, {}", size.width, size.height);
                size = PhysicalSize::new(450, 400);
            }
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                #[cfg(not(target_arch="wasm32"))]
                backends: wgpu::Backends::PRIMARY,
                #[cfg(target_arch="wasm32")]
                backends: wgpu::Backends::GL,
                ..Default::default()
            });
            let surface = instance.create_surface(window.clone()).unwrap();

            // let adapter = futures::executor::block_on(instance.request_adapter(
            //     &wgpu::RequestAdapterOptions {
            //         power_preference: wgpu::PowerPreference::default(),
            //         compatible_surface: Some(&surface),
            //         force_fallback_adapter: false,
            //     },
            // )).unwrap();
            let adapter = instance
                .enumerate_adapters(wgpu::Backends::all()).into_iter()
                .filter(|adapter| {
                    // check if this adapter supports our surface
                    adapter.is_surface_supported(&surface)
                }).next().unwrap();

            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                   required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )).unwrap();

            let surface_caps = surface.get_capabilities(&adapter);
            // Shader code in this tutorial assumes an sRGB surface texture. Using a different
            // one will result in all the colors coming out darker. If you want to support non
            // sRGB surfaces, you'll need to account for that when drawing to the frame.
            let surface_format = surface_caps.formats.iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            self.window = Some(window);
            self.render_state = Some(RenderState {
                surface,
                device,
                queue,
                config,
                size,
            });
            self.app.init();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                self.app.exit();
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                if self.render_state.is_some() {
                    self.update();
                    match self.draw() {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) =>
                            self.resize(self.render_state.as_ref().unwrap().size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            event_loop.exit();
                        },
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                    self.window.as_ref().unwrap().request_redraw();
                } else {
                    eprintln!("Cannot draw because not initialized");
                }
            },
            WindowEvent::Resized(physical_size) => {
                self.resize(physical_size);
            },
            _ => self.app.event(event),
        }
    }

    fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: StartCause) {
        let _ = (event_loop, cause);
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }
}

impl<'a, T: AppState + 'static> Window<'a, T> {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 && self.render_state.is_some() {
            let render_state = self.render_state.as_mut().unwrap();
            render_state.size = new_size;
            render_state.config.width = new_size.width;
            render_state.config.height = new_size.height;
            render_state.surface.configure(&render_state.device, &render_state.config);
            self.app.resize(cgmath::vec2(new_size.width, new_size.height));
        }
    }
    fn update(&mut self) {
    }
    fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        let render_state = self.render_state.as_mut().unwrap();
        let output = render_state.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = render_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        // submit will accept anything that implements IntoIter
        render_state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        return Ok(());
    }
    pub fn run(state: T) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn)
                    .expect("Couldn't initialize logger");
            } else {
                env_logger::init();
            }
        }

        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut window = Window {
            app: state,
            window: None,
            render_state: None,
        };

        event_loop.run_app(&mut window).unwrap();
    }
}

