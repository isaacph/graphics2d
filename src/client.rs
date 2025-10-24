use crate::{
    mat::{vec2, Mat4}, rrs::{r#abstract::RenderConstruct, Record, RecordSystem, Settings}, simple, square, texture::init_texture, textured, util::indirect_handles::WeakHandle, win
};

pub fn run() {
    win::run(Client::init);
}

pub mod render {
}

pub struct Client {
    ortho: Mat4,
    rrs: RecordSystem,
    simple_render: simple::Construct,
    square_render: square::Construct,
    textured_render: textured::Construct,
    texture: WeakHandle<textured::Texture>,
}

impl Client {
    pub fn init(rc: &mut win::RenderContext) -> Client {
        let mut rrs = RecordSystem::init();
        let simple_render = rrs.add(simple::Construct::init(rc));
        let square_render = rrs.add(square::Construct::init(rc));
        let mut textured_render = rrs.add(textured::Construct::init(rc));
        let texture_base = init_texture(rc, include_bytes!(env!("SAMPLE_IMAGE")), wgpu::FilterMode::Nearest)
            .expect("Could not load texture");
        let texture = textured_render.init_texture(rc, &mut rrs, texture_base).make_weak();
        return Client {
            ortho: Mat4::identity(),
            rrs,
            simple_render,
            square_render,
            textured_render,
            texture,
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

        let mut rr = Record::new();
        let matrix = Mat4::box2d(vec2(100.0 + regulate(time, 2.0) * 500.0, 100.0), vec2(100.0, 100.0));
        self.square_render.draw(rc, &mut rr, square::RenderParams { matrix, range: 0..3 });
        let matrix = Mat4::box2d(vec2(500.0 + regulate(time, 2.0) * 500.0, 300.0), vec2(100.0, 100.0));
        self.square_render.draw(rc, &mut rr, square::RenderParams { matrix, range: 3..6 });
        let matrix = Mat4::box2d(vec2(300.0 + regulate(time, 2.0) * 500.0, 300.0), vec2(100.0, 100.0));
        self.square_render.draw(rc, &mut rr, square::RenderParams { matrix, range: 3..6 });
        self.simple_render.draw(rc, &mut rr, ());
        let matrix = Mat4::box2d(vec2(100.0 + regulate(time, 2.0) * 500.0, 300.0), vec2(100.0, 100.0));
        self.square_render.draw(rc, &mut rr, square::RenderParams { matrix, range: 3..6 });
        let matrix = Mat4::box2d(vec2(200.0 + regulate(time, 2.0) * 500.0, 300.0), vec2(100.0, 100.0));
        self.square_render.draw(rc, &mut rr, square::RenderParams { matrix, range: 3..6 });
        let matrix = Mat4::box2d(vec2(400.0 + regulate(time, 2.0) * 500.0, 300.0), vec2(100.0, 100.0));
        self.square_render.draw(rc, &mut rr, square::RenderParams { matrix, range: 3..6 });
        let matrix = Mat4::box2d(vec2(400.0 + regulate(time, 2.0) * 500.0, 400.0), vec2(100.0, 100.0));
        self.textured_render.draw(rc, &mut rr, textured::RenderParams { matrix, texture: self.texture });

        let settings = Settings {
            projection: self.ortho,
        };
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

            self.rrs.render(rc, &mut rpass, &rr, &settings);
        }

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
