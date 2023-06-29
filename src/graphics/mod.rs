use std::{path::Path, collections::HashMap};

use cgmath::{vec2, vec4, Vector2};
use include_dir::{include_dir, Dir};
use itertools::Itertools;

use crate::{graphics::{texture::Texture, text::FontInfoContainer}, util::{world_to_world_screen_space, Depth, self}, window::Resources};
use self::{textured::{TextureRenderer, Instance}, text::{Font, FontRenderer, make_font_infos, default_characters}};

pub mod textured;
pub mod text;
pub mod texture;
pub mod grid;

// pub struct RenderPrereq<'a> {
//     pub device: &'a mut wgpu::Device,
//     pub queue: &'a mut wgpu::Queue,
//     pub surface: &'a mut wgpu::Surface,
// }
// 
// #[derive(Clone)]
// pub struct ResolveInstance {
//     pub position: cgmath::Vector2<f32>,
//     pub scale: cgmath::Vector2<f32>,
//     pub color: cgmath::Vector4<f32>,
//     pub overlaps: i32,
// }
// 
// impl From<ResolveInstance> for Instance {
//     fn from(val: ResolveInstance) -> Self {
//         Instance {
//             position: val.position,
//             scale: val.scale,
//             color: val.color
//         }
//     }
// }

// static RESOURCES: Dir<'_> = include_dir!("res");
// 
// pub fn load_bytes(res_path: &str) -> Option<(&[u8], &str)> {
//     RESOURCES.get_file(Path::new(res_path)).map(|file| (file.contents(), res_path))
// }
// 
// impl RenderEngine {
//     pub fn load_image(device: &wgpu::Device, queue: &wgpu::Queue, (bytes, name): (&[u8], &str)) -> Texture {
//         let diffuse_bytes = bytes;
//         let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
//         
//         Texture::from_image(device, queue, &diffuse_image, name, wgpu::FilterMode::Nearest).unwrap()
//     }
// 
//     pub fn load_fonts(device: &wgpu::Device, queue: &wgpu::Queue, (bytes, name): (&[u8], &str), sizes: &[f32]) -> Vec<Font> {
//         let font_info = make_font_infos(
//             bytes, sizes, default_characters().iter(), Some(&'\u{25A1}'), name.to_string()).unwrap();
//         font_info.into_iter().map(|info|
//             Font::make_from_info(device, queue, &info, wgpu::FilterMode::Linear).unwrap()
//         ).collect_vec()
//     }
// 
//     pub fn init(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) -> RenderEngine {
//         let mut texture_renderer = TextureRenderer::init(device, queue, config);
//         let mut ui_texture_renderer = TextureRenderer::init(device, queue, config);
//         let mut font_renderer = FontRenderer::new(device, queue, config).unwrap();
// 
//         // println!("{}", RESOURCES.files().map(|file| file.path().to_str()).flatten().join("\n"));
//         let mut fonts = Self::load_fonts(device, queue, load_bytes("NotoSerifJP-Regular.otf").unwrap(), &[24.0, 48.0]);
//         let font = fonts.pop().unwrap();
//         let small_font = fonts.pop().unwrap();
//         font_renderer.register_font(device, &font);
//         font_renderer.register_font(device, &small_font);
// 
//         let solid_texture = Texture::blank_texture(device, queue, "blank").unwrap();
//         ui_texture_renderer.add_texture(device, [&solid_texture].into_iter());
//         texture_renderer.add_texture(device, [&solid_texture].into_iter());
// 
//         // let wiz_walk_textures = (1..=12).map(|i|
//         //     Self::load_image(device, queue,
//         //         load_bytes(format!("walk_256/Layer {}.png", i).as_str()).unwrap())
//         //     ).collect_vec();
//         // texture_renderer.add_texture(device, wiz_walk_textures.iter());
// 
//         Self {
//             texture_renderer,
//             font_renderer,
//             ui_texture_renderer,
//             font,
//             small_font,
//             solid_texture,
//         }
//     }
// 
//     pub fn render(state: &mut State, resources: &mut Resources) -> Result<(), wgpu::SurfaceError> {
//         let engine = &mut state.render_engine;
//         let output = resources.surface.get_current_texture()?;
//         let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
// 
//         let mut encoder = resources.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
//             label: Some("Render Encoder"),
//         });
// 
//         {
//             let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                 label: Some("Render Pass"),
//                 color_attachments: &[
//                     Some(wgpu::RenderPassColorAttachment {
//                         view: &view,
//                         resolve_target: None,
//                         ops: wgpu::Operations {
//                             load: wgpu::LoadOp::Clear(
//                                 wgpu::Color {
//                                     r: 0.0,
//                                     g: 0.0,
//                                     b: 0.0,
//                                     a: 1.0,
//                                 }
//                             ),
//                             store: true,
//                         },
//                     })
//                 ],
//                 depth_stencil_attachment: None,
//             });
//             engine.texture_renderer.reset();
//             engine.font_renderer.reset();
//             engine.ui_texture_renderer.reset();
// 
//             let ui_camera = state.camera.get_ui_camera();
// 
//             // engine.texture_renderer.render(&mut resources.queue, &mut render_pass, &state.camera, world_draw).unwrap();
//             // engine.font_renderer.render(&engine.small_font, &resources.queue, &mut render_pass, &ui_camera, &names).unwrap();
// 
//             // render ui
//             let (background_instance, chatbox_text_instances) =
//                 state.chatbox.render();
//             engine.ui_texture_renderer.render(
//                 &mut resources.queue,
//                 &mut render_pass,
//                 &ui_camera,
//                 vec![
//                     (vec![background_instance], &engine.solid_texture)
//                 ]
//             )?;
//             // render chatbox
//             engine.font_renderer.render(
//                 &engine.font,
//                 &resources.queue,
//                 &mut render_pass,
//                 &ui_camera,
//                 &chatbox_text_instances
//             )?;
//         }
// 
//         // submit will accept anything that implements IntoIter
//         resources.queue.submit(std::iter::once(encoder.finish()));
//         output.present();
// 
//         Ok(())
//     }
// }
