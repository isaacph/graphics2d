use wgpu::util::DeviceExt;

use crate::{mat::{Mat4, Vec2}, rrs::{r#abstract::RenderConstruct, Entry, EntryDiscriminants, Record, Settings}, simple::{Simple, SimpleRenderer}, square::{Square, SquareRenderer}, win::RenderContext};
use std::{borrow::Cow, num::NonZero, ops::Range, str};

pub struct Depend {
    simple: Simple,
    square: Square,
}

pub struct DependRenderer {
    simple_render: SimpleRenderer,
    square_render: SquareRenderer,
}

#[derive(Debug)]
pub struct DependRenderParams {
    pub pos: Vec2,
}

// problems
//
// * multiple instances of the same resources
// * requires new entry types just for conditional/combined rendering of others
// * otherwise requires every single render type to be individually aware and reimplementing
//   filtering and transforming features
//      * how to even slot in generic filtering/transforming?
//      you could make the renderers abstract and accept a method that decides
//      filtering/transforming based on entry/settings
//      i'd like to have multiple of these abstract renderers share the same resources. is that
//      possible?
//          * requires refcell or?
//      so I know i'd like to call these "variants" of renderers


impl Depend {
    pub fn init(rc: &mut RenderContext) -> Self {
        return Depend {
            simple: Simple::init(rc),
            square: Square::init(rc),
        };
    }
}

impl RenderConstruct<Entry, EntryDiscriminants, Settings> for Depend {
    type Renderer = DependRenderer;
    type DrawParam = DependRenderParams;

    fn init_renderer(&mut self) -> DependRenderer {
        DependRenderer {
            simple_render: self.simple.init_renderer(),
            square_render: self.square.init_renderer(),
        }
    }

    fn draw(&mut self, _rc: &mut RenderContext, record: &mut Record, data: DependRenderParams) {
        record.entries.push(Entry::Depend(data));
    }
}

impl crate::rrs::r#abstract::Renderer<Entry, EntryDiscriminants> for DependRenderer {
    type Settings = Settings;

    fn discriminant(&self) -> EntryDiscriminants {
        EntryDiscriminants::Square
    }

    fn pre_render(&mut self, rc: &mut RenderContext, record: &Record, _: &Settings) {
    }

    fn render(&mut self, rc: &mut RenderContext, rpass: &mut wgpu::RenderPass, entry: &Entry, _: &Settings) {
    }

    fn post_render(&mut self, _rc: &mut RenderContext, _: &Record, _: &Settings) {
    }
}


