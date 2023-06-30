use cgmath::{Vector4, Vector2};
use clipboard::{ClipboardContext, ClipboardProvider};
use itertools::Itertools;
use winit::{event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode, MouseScrollDelta}, dpi::PhysicalPosition};

use crate::{graphics::{text::{FontMetricsInfo, FontInfoContainer, BaseFontInfoContainer}, textured}, util::clampi};

pub struct Chatbox {
    font_info: FontMetricsInfo,
    visible_lines: i32,
    line_height: f32,
    history_length: i32,
    typing: String,
    history: Vec<String>,
    history_split: Vec<String>,
    width: f32,
    height: f32,
    flicker_timer: f32,
    typing_flicker: bool,
    fade_timer: f32,
    scroll: i32, // a number from 0 to max_scroll for the scroll value
    max_scroll: i32, // cache the maximum we calculated you could scroll
    scroll_float: f32, // the extra partial scroll value from scrolling that is not aligned
    pub scroll_speed: f32,
}

pub const BAR_FLICKER_TIME: f32 = 0.6;
pub const FADE_START_TIME: f32 = 3.0;
pub const FADE_TIME: f32 = 1.0;

impl Chatbox {
    pub fn new(font_info: FontMetricsInfo, line_height: f32, history_length: i32) -> Self {
        assert!(history_length >= 0 && line_height >= 0.0);
        Chatbox {
            font_info,
            visible_lines: 0,
            line_height,
            history_length,
            typing: String::new(),
            history: Vec::new(),
            history_split: Vec::new(),
            width: 800.0,
            height: 0.0,
            flicker_timer: 0.0,
            typing_flicker: false,
            fade_timer: f32::MAX,
            scroll: 0,
            max_scroll: 0,
            scroll_float: 0.0,
            scroll_speed: 0.05,
        }
    }

    pub fn line_height(&self) -> f32 {
        self.line_height
    }
    pub fn visible_lines(&self) -> i32 {
        self.visible_lines
    }
    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn set_scroll(&mut self, scroll: i32) {
        self.scroll = i32::min(i32::max(0, scroll), self.max_scroll);
        self.scroll_float = 0.0;
        self.regen_split();
    }
    pub fn max_scroll(&self) -> i32 {
        self.max_scroll
    }

    pub fn resize(&mut self, width: f32, visible_lines: i32) {
        assert!(visible_lines >= 0 && width >= 0.0);
        self.width = width;
        self.visible_lines = visible_lines;
        self.height = visible_lines as f32 * self.line_height + self.line_height / 4.0;
        self.regen_split()
    }

    fn regen_split(&mut self) {
        // resolve scrolling
        let line_change = f32::round(self.scroll_float * self.scroll_speed) as i32;
        self.scroll_float -= line_change as f32 / self.scroll_speed;
        self.scroll = i32::min(self.max_scroll, i32::max(0, self.scroll + line_change));

        let wrap = self.font_info.split_lines(&self.history.join("\n"), Some(self.width)).collect_vec();
        let pos = wrap.len() as i32 - self.visible_lines - self.scroll as i32;
        let pos = clampi(pos, 0, clampi(wrap.len() as i32 - self.visible_lines, 0, wrap.len() as i32));
        let wrap_range = &wrap[
            pos as usize..
            clampi(pos + self.visible_lines, 0, wrap.len() as i32) as usize];
        self.max_scroll = std::cmp::max(0, wrap.len() as i32 - self.visible_lines);
        self.history_split = wrap_range.into_iter().cloned().collect();
    }

    pub fn println(&mut self, line: &str) {
        println!("chat println: {}", line);
        let split = self.font_info.split_lines(line, None);
        // take the last x lines
        let add = split.collect_vec();
        let add = &add[std::cmp::max(0, add.len() as i32 - self.history_length) as usize..add.len()];
        let history_remove = 
            std::cmp::max(0, self.history.len() as i32 - (self.history_length - add.len() as i32)) as usize;
        self.history.drain(0..history_remove);
        self.history.extend(add.iter().cloned());

        self.regen_split();
        self.fade_timer = 0.0;
    }

    fn get_visible_history_empty_lines(&self) -> i32 {
        std::cmp::max(0, self.visible_lines - self.history_split.len() as i32)
    }

    pub fn get_visible_history(&self) -> &Vec<String> {
        // let mut vec = Vec::new();
        // for i in (std::cmp::max(0, self.history.len() as i32 - self.visible_lines) as usize)..self.history.len() {
        //     vec.push(self.history[i].as_str());
        // }
        // vec
        &self.history_split
    }

    pub fn get_typing(&self) -> &String {
        &self.typing
    }

    pub fn add_typing(&mut self, c: char) {
        self.typing.push(c);
    }

    pub fn add_typing_lines(&mut self, s: &str) {
        self.typing += s;
    }

    pub fn remove_typing(&mut self, count: i32) {
        assert!(count >= 0);
        for _ in 0..count {
            if self.typing.is_empty() {
                break
            }
            self.typing.pop();
        }
    }

    pub fn erase_typing(&mut self) {
        self.typing.clear();
    }

    pub fn set_typing_flicker(&mut self, typing_flicker: bool) {
        self.typing_flicker = typing_flicker;
        self.flicker_timer = 0.0;
        self.fade_timer = 0.0;
    }

    pub fn update(&mut self, delta_time: f32) {
        self.fade_timer += delta_time;
        if self.typing_flicker {
            self.flicker_timer += delta_time;
            while self.flicker_timer > BAR_FLICKER_TIME {
                self.flicker_timer -= BAR_FLICKER_TIME;
            }
        }
    }

    pub fn render(&self) -> (textured::Instance, Vec<(String, Vector2<f32>, Vector4<f32>)>) {
        let is_fade = self.fade_timer > FADE_START_TIME && !self.typing_flicker;
        let mut fade = 1.0;
        if is_fade {
            fade = 1.0 - f32::max(0.0, (self.fade_timer - FADE_START_TIME) / FADE_TIME);
        }

        let color = Vector4::new(1.0, 1.0, 1.0, 1.0) * fade;
        let background_color = Vector4::new(0.0, 0.0, 0.0, 0.6) * fade;
        let position = Vector2::new(0.0, 0.0);

        let effective_height = self.height + if self.typing_flicker { self.line_height } else { 0.0 };
        let background_instance = textured::Instance {
            color: background_color,
            position: Vector2::new(position.x + self.width / 2.0, position.y + effective_height / 2.0),
            scale: Vector2::new(self.width, effective_height),
        };
        
        let start = Vector2::new(
            position.x,
            position.y + self.line_height * (self.get_visible_history_empty_lines() + 1) as f32
        );
        let (pos, mut instances) = self.get_visible_history().iter().fold((start, vec![]), |(mut pos, mut instances), line| {
            instances.push((line.to_string(), pos, color));
            pos += Vector2::new(0.0, self.line_height);
            (pos, instances)
        });

        if self.typing_flicker {
            let typing_line = "> ".to_string() +
                &if self.flicker_timer > BAR_FLICKER_TIME / 2.0 && self.typing_flicker {
                    self.typing.to_owned() + "|"
                } else {
                    self.typing.to_owned()
                };
            instances.push((typing_line, pos, color));
        }
        (background_instance, instances)
    }

    pub fn focus(&mut self) {
        self.set_typing_flicker(true);
        self.set_scroll(0);
    }

    pub fn unfocus(&mut self) {
        self.set_typing_flicker(false);
    }

    pub fn process_scroll(&mut self, scroll_y: f32) {
        self.scroll_float += scroll_y;
        self.regen_split();
    }

    pub fn receive_focused_event(&mut self, event: &WindowEvent, clipboard: &mut ClipboardContext) -> ReceiveResult {
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
                        self.set_typing_flicker(false);
                        return ReceiveResult::Relinquish;
                    },
                    VirtualKeyCode::Return => {
                        if self.get_typing().is_empty() {
                            self.set_typing_flicker(false);
                            return ReceiveResult::Relinquish;
                        } else {
                            let typing = self.get_typing().clone();
                            // self.chatbox.println(&typing);
                            self.erase_typing();
                            return ReceiveResult::Command(typing);
                        }
                    },
                    VirtualKeyCode::V => {
                        if modifiers.ctrl() {
                            // CTRL+V
                            let res = clipboard.get_contents();
                            if let Ok(clipboard) = res {
                                self.add_typing_lines(&clipboard);
                            }
                            else if let Err(err) = res {
                                self.println(&("Error pasting: ".to_string() + &err.to_string()));
                            }
                            return ReceiveResult::Consumed;
                        }
                    },
                    _ => ()
                }
                return ReceiveResult::Consumed;
            },
            WindowEvent::ReceivedCharacter(c) => {
                if c == '\x08' { // backspace
                    self.remove_typing(1);
                    return ReceiveResult::Consumed;
                } else if !self.font_info.is_char_valid(&c) {
                    // ignore invalid characters
                    // this includes keycodes generated from like Ctrl + V
                } else {
                    self.add_typing(c);
                    return ReceiveResult::Consumed;
                }
            },
            // grab mouse wheel events
            WindowEvent::MouseWheel {
                delta,
                phase: _,
                ..
            } => {
                let (_dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(dx, dy) => {
                        // we're just assuming a "line" is about 32 px
                        (dx as f32 * 32.0, dy as f32 * 32.0)
                    },
                    MouseScrollDelta::PixelDelta(PhysicalPosition {x: dx, y: dy}) => {
                        (dx as f32, dy as f32)
                    },
                };
                // response to mouse wheel input
                // this just scrolls the chat
                self.process_scroll(dy);
                return ReceiveResult::Consumed;
            },
            _ => ()
        };
        return ReceiveResult::Ignored;
    }
}

pub enum ReceiveResult {
    Ignored,
    Consumed,
    Relinquish,
    Command(String),
}

impl ReceiveResult {
    pub fn consumed(&self) -> bool {
        match self {
            Self::Consumed => true,
            Self::Relinquish => true,
            Self::Command(_) => true,
            _ => false,
        }
    }
    pub fn relinquished(&self) -> bool {
        match self {
            Self::Relinquish => true,
            _ => false,
        }
    }
    pub fn get_command(&self) -> Option<String> {
        match self {
            Self::Command(cmd) => Some(cmd.clone()),
            _ => None,
        }
    }
}
