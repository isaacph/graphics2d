#![windows_subsystem = "windows"]
use cgmath::vec2;
use cgmath::{Vector2, Zero};
use chatbox::Chatbox;
use clipboard::{ClipboardContext, ClipboardProvider};
use graphics::{RenderEngine, text::BaseFontInfoContainer};
use instant::Instant;
use itertools::Itertools;
use window::{Resources, StateTrait};
use winit::dpi::PhysicalPosition;
use std::collections::HashSet;
use winit::{
    event::*,
    event_loop::EventLoop,
};
use winit::window::Window;

// how to start from local program
fn main() {
    window::run(State::init);
}

// how to start from website
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
async fn wasm_main() {
    window::run_async(State::init);
}

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FocusMode {
    Default, Chatbox
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameState {
    Game, Editor
}

pub const DEFAULT_ZOOM: f32 = 4.0;

pub struct State {
    render_engine: RenderEngine,

    camera: camera::Camera,
    camera_controller: camera::CameraController,

    last_frame: Instant,

    clipboard: ClipboardContext,

    pub input_state: InputState,
    pub mouse_pos_view: Vector2<f32>,

    pub chatbox: Chatbox,
    pub chatbox_scroll: f32,
    pub focus_mode: FocusMode,
    pub game_state: GameState,
}

pub struct InputState {
    pub key_down: HashSet<VirtualKeyCode>,
    pub key_pos_edge: HashSet<VirtualKeyCode>,
    pub key_neg_edge: HashSet<VirtualKeyCode>,
    pub mouse_pos_edge: HashSet<MouseButton>,
    pub mouse_position: Vector2<f32>,
    pub commands: Vec<String>,
    pub edit: bool,
    pub mouse_wheel: Vector2<f32>,
}

impl State {
    pub fn init(resources: &mut Resources) -> Box<dyn StateTrait> {
        // // load a texture - happy tree
        // let diffuse_bytes = include_bytes!("happy-tree.png");
        // let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        // let diffuse_texture =
        //     texture::Texture::from_image(&device, &queue, &diffuse_image, "happy-tree").unwrap();

        // // load a texture - keyboard
        // let diffuse_bytes_2 = include_bytes!("keyboard.jpg");
        // let diffuse_image_2 = image::load_from_memory(diffuse_bytes_2).unwrap();
        // let diffuse_texture_2 =
        //     texture::Texture::from_image(&device, &queue, &diffuse_image_2, "keyboard").unwrap();

        // camera
        let camera = camera::Camera::new(cgmath::Vector2::new(resources.size.width, resources.size.height), DEFAULT_ZOOM);
        let camera_controller = camera::CameraController::new(1.0);

        let render_engine = RenderEngine::init(&resources.device, &resources.queue, &resources.config);
        let chatbox = Chatbox::new(render_engine.font.get_metrics_info(), 42.0, 40);

        // create example characters to display
        // let mut char_id_gen = CharacterIDGenerator::new();
        // let proj_id_gen = ProjectileIDGenerator::new();
        // let player_id = Some(world.instantiate(ice_wiz(), char_id_gen.generate(), vec3(0.0, 0.0, 0.0)).unwrap());
        // let _minion = world.instantiate(caster_minion(), char_id_gen.generate(), vec3(1.0, 0.0, 0.0)).unwrap();
        // let _minion2 = world.instantiate(caster_minion(), char_id_gen.generate(), vec3(1.5, 1.0, 0.0)).unwrap();

        let mut state = Self {
            render_engine,
            camera,
            camera_controller,
            last_frame: Instant::now(),
            input_state: InputState {
                key_down: HashSet::new(),
                key_pos_edge: HashSet::new(),
                key_neg_edge: HashSet::new(),
                mouse_pos_edge: HashSet::new(),
                mouse_position: Vector2::zero(),
                commands: vec![],
                edit: true,
                mouse_wheel: Vector2::new(0.0, 0.0),
            },
            mouse_pos_view: Vector2::zero(),
            chatbox,
            chatbox_scroll: 0.0,
            focus_mode: FocusMode::Default,
            game_state: GameState::Game,
            clipboard: ClipboardProvider::new().unwrap(),
        };
        
        Box::new(state)
    }
}

impl StateTrait for State {
    fn resize(&mut self, _window: &Window, _resources: &mut Resources, new_size: winit::dpi::PhysicalSize<u32>) {
        self.camera.window_size = cgmath::Vector2::new(new_size.width, new_size.height);
        self.chatbox.resize(new_size.width as f32, (new_size.height as f32 * 1.0 / 3.0 / self.chatbox.line_height()) as i32)
    }

    fn input(&mut self, _window: &Window, _resources: &mut Resources, event: &WindowEvent) -> bool {
        // overrides from chatbox mode
        if self.focus_mode == FocusMode::Chatbox {
            match *event {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(key),
                        modifiers, // the alternative doesn't even work on the web atm so we're
                                   // using this version despite deprecation
                        ..
                    },
                    ..
                } => {
                    match key {
                        VirtualKeyCode::Escape => {
                            self.focus_mode = FocusMode::Default;
                            self.chatbox.set_typing_flicker(false);
                        },
                        VirtualKeyCode::Return => {
                            if self.chatbox.get_typing().is_empty() {
                                self.focus_mode = FocusMode::Default;
                                self.chatbox.set_typing_flicker(false);
                            } else {
                                let typing = self.chatbox.get_typing().clone();
                                // self.chatbox.println(&typing);
                                self.chatbox.erase_typing();
                                self.input_state.commands.push(typing);
                            }
                        },
                        VirtualKeyCode::V => {
                            if modifiers.ctrl() {
                                // CTRL+V
                                let res = self.clipboard.get_contents();
                                if let Ok(clipboard) = res {
                                    self.chatbox.add_typing_lines(&clipboard);
                                }
                                else if let Err(err) = res {
                                    self.chatbox.println(&("Error pasting: ".to_string() + &err.to_string()));
                                }
                            }
                        },
                        _ => {
                        }
                    }
                    return true
                },
                WindowEvent::ReceivedCharacter(c) => {
                    if c == '\x08' { // backspace
                        self.chatbox.remove_typing(1);
                        return true
                    } else if !self.render_engine.font.is_char_valid(&c) {
                        // ignore invalid characters
                        // this includes keycodes generated from like Ctrl + V
                    } else {
                        self.chatbox.add_typing(c);
                        return true
                    }
                },
                // grab mouse wheel events
                WindowEvent::MouseWheel {
                    delta,
                    phase: _,
                    ..
                } => {
                    let (dx, dy) = match delta {
                        MouseScrollDelta::LineDelta(dx, dy) => {
                            // we're just assuming a "line" is about 32 px
                            (dx as f32 * 32.0, dy as f32 * 32.0)
                        },
                        MouseScrollDelta::PixelDelta(PhysicalPosition {x: dx, y: dy}) => {
                            (dx as f32, dy as f32)
                        },
                    };
                    self.input_state.mouse_wheel += Vector2::new(dx, dy);
                    // response to mouse wheel input
                    // this just scrolls the chat
                    let scroll_y = self.input_state.mouse_wheel.y;
                    let scroll_approx = ((scroll_y - self.chatbox_scroll) / self.chatbox.line_height()) as i32;
                    self.chatbox.set_scroll(scroll_approx);
                    if scroll_approx < 0 {
                        self.chatbox_scroll = scroll_y;
                    } else if scroll_approx > self.chatbox.max_scroll() as i32 {
                        self.chatbox_scroll = scroll_y
                            - self.chatbox.line_height() * self.chatbox.max_scroll() as f32;
                    }
                    return true
                },
                _ => ()
            };
        }
        // regular mode
        // only two inputs that are actually capable of changing the model are S and F
        let relevant_inputs = {
            use VirtualKeyCode::*;
            vec![S, F]
        };
        if !self.camera_controller.process_events(event) {
            match *event {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state,
                        virtual_keycode:
                        Some(key),
                        ..
                    },
                    ..
                } => match key {
                    VirtualKeyCode::Return => if state == ElementState::Pressed {
                        self.focus_mode = FocusMode::Chatbox;
                        self.chatbox.set_typing_flicker(true);
                        self.chatbox_scroll = self.input_state.mouse_wheel.y;
                        self.chatbox.set_scroll(0);
                        true
                    } else { true },
                    _ => {
                        if relevant_inputs.contains(&key) {
                            match state {
                                ElementState::Pressed => {
                                    self.input_state.key_down.insert(key);
                                    self.input_state.key_pos_edge.insert(key);
                                },
                                ElementState::Released => {
                                    self.input_state.key_down.remove(&key);
                                    self.input_state.key_neg_edge.insert(key);
                                },
                            };
                            true
                        } else { false }
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    let position = vec2(position.x as f32, position.y as f32);
                    self.mouse_pos_view = position;
                    self.input_state.mouse_position = self.camera.view_to_world_pos(position);
                    true
                },
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button,
                    ..
                } => {
                    self.input_state.mouse_pos_edge.insert(button);
                    true
                },
                _ => false,
            }
        } else {
            true
        }
    }

    fn update(&mut self, _window: &Window, _resources: &mut Resources) -> bool {
        // timing
        let frame = Instant::now();
        let delta_time = ((frame - self.last_frame).as_nanos() as f64 / 1000000000.0) as f32;
        self.last_frame = frame;

        // input
        if self.input_state.commands.iter().map(|command| {
            let split = &command.split(' ').collect::<Vec<_>>()[..];
            match split {
                ["exit"] => return Ok(true),
                ["edit"] => self.game_state = GameState::Editor,
                ["game"] => self.game_state = GameState::Game,
                ["echo"] => {
                    self.chatbox.println("Empty echo");
                },
                ["echo", _, ..] => {
                    self.chatbox.println(&command[split[0].len()+1..]);
                },
                _ => self.chatbox.println("Unknown command or incorrect arguments"),
            }
            Ok(false)
        }).collect_vec().into_iter().filter_map(|r: Result<bool, String>| r.map_err(|err|
            self.chatbox.println(format!("Error running command -- {}", err).as_str())
        ).ok()).any(|x| x) {
            return true
        }
        self.input_state.commands.clear();

        // camera update
        // let mut player_position = None;
        // if let Some(player_id) = self.player_id {
        //     if let Some(player_cid) = self.player_data.get_player(&player_id).unwrap().selected_char {
        //         if let Some(ch) = self.world.characters.get(&player_cid) {
        //             if let Some(props) = self.world.char_props.get(&player_cid) {
        //                 player_position = Some(ch.position - props.center_offset);
        //             }
        //         }
        //     }
        // }
        // self.camera_controller.update_camera(delta_time, &mut self.camera, player_position.as_ref());

        self.chatbox.update(delta_time);
        
        // clear inputs
        self.input_state.key_pos_edge.clear();
        self.input_state.key_neg_edge.clear();
        self.input_state.mouse_pos_edge.clear();
        false
    }

    fn render(&mut self, _window: &Window, resources: &mut Resources) -> Result<(), wgpu::SurfaceError> {
        RenderEngine::render(self, resources)
    }

    fn close(&mut self, _window: &Window, _resources: &mut Resources) {
    }
}
