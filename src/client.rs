use crate::{mat::{vec2, Mat4}, simple, square, win};

pub fn run() {
    win::run(Client::init);
}

pub struct Client {
    ortho: Mat4,
    simple_render: simple::Simple,
    square_render: square::Square,
}

impl Client {
    pub fn init(rc: &mut win::RenderContext) -> Client {
        return Client {
            ortho: Mat4::identity(),
            simple_render: simple::Simple::init(rc),
            square_render: square::Square::init(rc),
        };
    }
}

fn regulate(f: f32, bound: f32) -> f32 {
    let x = f.rem_euclid(bound * 2.0);
    if x > bound {
        return 2.0 * bound - x;
    } else {
        return x;
    }
}

impl win::Client for Client {
    fn draw(&mut self, rc: &mut win::RenderContext) {
        let time = rc.time();
        let output = match rc.surface.get_current_texture() {
            Ok(x) => x,
            Err(_err) => return,
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = rc.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            self.simple_render.draw(rc, &mut rpass);
            let mat = self.ortho * Mat4::box2d(vec2(100.0 + regulate(time, 2.0) * 500.0, 100.0), vec2(100.0, 100.0));
            self.square_render.draw(rc, &mut rpass, &mat);
        }

        // submit will accept anything that implements IntoIter
        rc.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn resize(&mut self, _rc: &mut win::RenderContext, size: winit::dpi::PhysicalSize<u32>) {
        self.ortho = Mat4::ortho(size);
    }

    fn handle_event(&mut self, rc: &mut win::RenderContext, event: &winit::event::WindowEvent) -> win::EventState {
        match event {
            winit::event::WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _, } => {
                match event.logical_key {
                    winit::keyboard::Key::Named(named_key) => match named_key {
                        winit::keyboard::NamedKey::Escape => rc.exit(),
                        _ => (),
                    },
                    _ => (),
                };
                return win::EventState::Consumed;
            },
            _ => win::EventState::Skipped,
        }
    }
}
