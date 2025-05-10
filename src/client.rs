use crate::{simple, square, win};

pub fn run() {
    win::run(Client::init);
}

pub struct Client {
    simple_render: simple::Simple,
    square_render: square::Square,
}

impl Client {
    pub fn init(rc: &mut win::RenderContext) -> Client {
        return Client {
            simple_render: simple::Simple::init(rc),
            square_render: square::Square::init(rc),
        };
    }
}

impl win::Client for Client {
    fn draw(&mut self, rc: &mut win::RenderContext) {
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
            self.square_render.draw(rc, &mut rpass);
        }

        // submit will accept anything that implements IntoIter
        rc.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn resize(&mut self, _rc: &mut win::RenderContext, _size: winit::dpi::PhysicalSize<u32>) {
    }
}
