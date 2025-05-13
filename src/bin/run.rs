// #![windows_subsystem = "windows"]
// use cgmath::vec2;
// use cgmath::{Vector2, Zero};
// use graphics::camera;
// use graphics::chatbox::Chatbox;
// use clipboard::{ClipboardContext, ClipboardProvider};
// use graphics::text::{BaseFontInfoContainer, FontRenderer, make_font_infos, default_characters, Font};
// use graphics::texture::Texture;
// use graphics::textured::TextureRenderer;
// use include_dir::Dir;
// use instant::Instant;
// use itertools::Itertools;
// use graphics::window::{Resources, StateTrait, self};
// use wgpu::StoreOp;
// use std::collections::HashSet;
// use std::path::Path;
// use winit::event::*;
// use winit::window::Window;
// use winit::keyboard::{PhysicalKey, KeyCode};
// 
// // how to start from local program
// fn main() {
//     window::run(State::init);
// }
// 
// // how to start from website
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::prelude::wasm_bindgen;
// #[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
// async fn wasm_main() {
//     window::run_async(State::init);
// }
// 
// #[cfg(target_arch="wasm32")]
// use wasm_bindgen::prelude::*;
// 
// #[derive(Copy, Clone, PartialEq, Eq)]
// pub enum FocusMode {
//     Default, Chatbox
// }
// 
// #[derive(Copy, Clone, PartialEq, Eq)]
// pub enum GameState {
//     Game, Editor
// }
// 
// pub const DEFAULT_ZOOM: f32 = 4.0;
// 
// pub struct State {
//     render_engine: RenderEngine,
// 
//     camera: camera::Camera,
//     camera_controller: camera::CameraController,
// 
//     last_frame: Instant,
// 
//     // clipboard: ClipboardContext,
// 
//     pub input_state: InputState,
//     pub mouse_pos_view: Vector2<f32>,
// 
//     pub chatbox: Chatbox,
//     pub focus_mode: FocusMode,
//     pub game_state: GameState,
// }
// 
// pub struct InputState {
//     pub key_down: HashSet<KeyCode>,
//     pub key_pos_edge: HashSet<KeyCode>,
//     pub key_neg_edge: HashSet<KeyCode>,
//     pub mouse_down: HashSet<MouseButton>,
//     pub mouse_pos_edge: HashSet<MouseButton>,
//     pub mouse_neg_edge: HashSet<MouseButton>,
//     pub mouse_position: Vector2<f32>,
//     pub commands: Vec<String>,
//     pub edit: bool,
//     pub mouse_wheel: Vector2<f32>,
// }
// 
// impl State {
//     pub fn init(resources: &mut Resources) -> Box<dyn StateTrait> {
//         let camera = camera::Camera::new(cgmath::Vector2::new(resources.size.width, resources.size.height), DEFAULT_ZOOM);
//         let camera_controller = camera::CameraController::new(1.0);
// 
//         let render_engine = RenderEngine::init(&resources.device, &resources.queue, &resources.config);
//         let chatbox = Chatbox::new(render_engine.font.get_metrics_info(), 42.0, 40);
// 
//         let state = Self {
//             render_engine,
//             camera,
//             camera_controller,
//             last_frame: Instant::now(),
//             input_state: InputState {
//                 key_down: HashSet::new(),
//                 key_pos_edge: HashSet::new(),
//                 key_neg_edge: HashSet::new(),
//                 mouse_down: HashSet::new(),
//                 mouse_pos_edge: HashSet::new(),
//                 mouse_neg_edge: HashSet::new(),
//                 mouse_position: Vector2::zero(),
//                 commands: vec![],
//                 edit: true,
//                 mouse_wheel: Vector2::new(0.0, 0.0),
//             },
//             mouse_pos_view: Vector2::zero(),
//             chatbox,
//             focus_mode: FocusMode::Default,
//             game_state: GameState::Game,
//         };
//         
//         Box::new(state)
//     }
// }
// 
// impl StateTrait for State {
//     fn resize(&mut self, _window: &Window, _resources: &mut Resources, new_size: winit::dpi::PhysicalSize<u32>) {
//         self.camera.window_size = cgmath::Vector2::new(new_size.width, new_size.height);
//         self.chatbox.resize(new_size.width as f32, (new_size.height as f32 * 1.0 / 3.0 / self.chatbox.line_height()) as i32)
//     }
// 
//     fn input(&mut self, _window: &Window, resources: &mut Resources, event: &WindowEvent) -> bool {
//         // overrides from chatbox mode
//         if self.focus_mode == FocusMode::Chatbox {
//             let result = self.chatbox.receive_focused_event(event, &resources.modifiers);
//             if result.relinquished() {
//                 self.focus_mode = FocusMode::Default;
//             }
//             if let Some(cmd) = result.get_command() {
//                 self.input_state.commands.push(cmd);
//             }
//             if result.consumed() {
//                 return true;
//             }
//         } else if self.focus_mode == FocusMode::Default {
//             // regular mode
//             // only two inputs that are actually capable of changing the model are S and F
//             let relevant_inputs = {
//                 use KeyCode::*;
//                 vec![KeyS, KeyF]
//             };
//             if !self.camera_controller.process_events(event) {
//                 match *event {
//                     WindowEvent::KeyboardInput {
//                         event: KeyEvent {
//                             physical_key: PhysicalKey::Code(key),
//                             state, ..
//                         },
//                         ..
//                     } => match key {
//                         KeyCode::Enter => if state == ElementState::Pressed {
//                             self.focus_mode = FocusMode::Chatbox;
//                             self.chatbox.focus();
//                             return true
//                         } else { return true },
//                         _ => {
//                             if relevant_inputs.contains(&key) {
//                                 match state {
//                                     ElementState::Pressed => {
//                                         self.input_state.key_down.insert(key);
//                                         self.input_state.key_pos_edge.insert(key);
//                                     },
//                                     ElementState::Released => {
//                                         self.input_state.key_down.remove(&key);
//                                         self.input_state.key_neg_edge.insert(key);
//                                     },
//                                 };
//                                 return true
//                             }
//                         }
//                     },
//                     WindowEvent::CursorMoved { position, .. } => {
//                         let position = vec2(position.x as f32, position.y as f32);
//                         self.mouse_pos_view = position;
//                         self.input_state.mouse_position = self.camera.view_to_world_pos(position);
//                         return true
//                     },
//                     WindowEvent::MouseInput {
//                         state,
//                         button,
//                         ..
//                     } => match state {
//                         ElementState::Pressed => {
//                             self.input_state.mouse_pos_edge.insert(button);
//                             self.input_state.mouse_down.insert(button);
//                             return true
//                         },
//                         ElementState::Released => {
//                             self.input_state.mouse_neg_edge.insert(button);
//                             self.input_state.mouse_down.remove(&button);
//                             return true
//                         },
//                     },
//                     _ => (),
//                 };
//             } else {
//                 return true
//             }
//         }
//         return false
//     }
// 
//     fn update(&mut self, _window: &Window, _resources: &mut Resources) -> bool {
//         // timing
//         let frame = Instant::now();
//         let delta_time = ((frame - self.last_frame).as_nanos() as f64 / 1000000000.0) as f32;
//         self.last_frame = frame;
// 
//         // commands
//         if self.input_state.commands.iter().map(|command| {
//             let split = &command.split(' ').collect::<Vec<_>>()[..];
//             match split {
//                 ["exit"] => return Ok(true),
//                 ["edit"] => self.game_state = GameState::Editor,
//                 ["game"] => self.game_state = GameState::Game,
//                 ["echo"] => {
//                     self.chatbox.println("Empty echo");
//                 },
//                 ["echo", _, ..] => {
//                     self.chatbox.println(&command[split[0].len()+1..]);
//                 },
//                 ["scrollspeed", speed_str] => {
//                     let speed_res = speed_str.parse();
//                     match speed_res {
//                         Ok(speed) => {
//                             self.chatbox.scroll_speed = speed;
//                             self.chatbox.println(format!("Set scroll speed to {}", speed).as_str());
//                         },
//                         Err(err) => {
//                             return Err(err.to_string());
//                         },
//                     }
//                 },
//                 _ => self.chatbox.println("Unknown command or incorrect arguments"),
//             }
//             Ok(false)
//         }).collect_vec().into_iter().filter_map(|r: Result<bool, String>| r.map_err(|err|
//             self.chatbox.println(format!("Error running command -- {}", err).as_str())
//         ).ok()).any(|x| x) {
//             return true
//         }
//         self.input_state.commands.clear();
// 
//         // camera update
//         self.camera_controller.update_camera(delta_time, &mut self.camera);
// 
//         // chatbox update
//         self.chatbox.update(delta_time);
//         
//         // clear inputs after we've read them
//         self.input_state.key_pos_edge.clear();
//         self.input_state.key_neg_edge.clear();
//         self.input_state.mouse_pos_edge.clear();
//         self.input_state.mouse_neg_edge.clear();
//         false
//     }
// 
//     fn render(&mut self, _window: &Window, resources: &mut Resources) -> Result<(), wgpu::SurfaceError> {
//         RenderEngine::render(self, resources)
//     }
// 
//     fn close(&mut self, _window: &Window, _resources: &mut Resources) {
//     }
// }
// 
// static RESOURCES: Dir<'_> = include_dir::include_dir!("res");
// 
// pub fn load_bytes(res_path: &str) -> Option<&[u8]> {
//     RESOURCES.get_file(Path::new(res_path)).map(|file| file.contents())
// }
// 
// pub struct RenderEngine {
//     pub texture_renderer: TextureRenderer,
//     pub ui_texture_renderer: TextureRenderer,
//     pub font_renderer: FontRenderer,
//     pub font: Font,
//     pub small_font: Font,
//     pub solid_texture: Texture,
// }
// 
// impl RenderEngine {
//     pub fn init(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) -> RenderEngine {
//         let mut texture_renderer = TextureRenderer::init(device, queue, config);
//         let mut ui_texture_renderer = TextureRenderer::init(device, queue, config);
//         let mut font_renderer = FontRenderer::new(device, queue, config).unwrap();
// 
//         // println!("{}", RESOURCES.files().map(|file| file.path().to_str()).flatten().join("\n"));
//         let font_info = make_font_infos(
//             load_bytes("NotoSerifJP-Regular.otf").unwrap(),
//             &[24.0, 48.0],
//             default_characters().iter(),
//             Some(&'\u{25A1}'),
//             "NotoSerifJP-Regular.otf".to_string()).unwrap();
//         let mut fonts = font_info.into_iter().map(|info|
//             Font::make_from_info(device, queue, &info, wgpu::FilterMode::Linear).unwrap()
//         ).collect_vec();
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
//                             store: StoreOp::Store,
//                         },
//                     })
//                 ],
//                 depth_stencil_attachment: None,
//                 timestamp_writes: None,
//                 occlusion_query_set: None,
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

fn main() {
}
