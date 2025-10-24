use strum::EnumDiscriminants;
use std::collections::HashMap;
use crate::win::RenderContext;
use crate::mat::Mat4;

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
pub enum Entry {
    Simple,
    Square(crate::square::RenderParams),
    Textured(crate::textured::RenderParams),
}

pub struct Settings {
    pub projection: Mat4,
}

#[derive(Debug)]
pub enum Update<'a> {
    Empty,
    Args(UpdateArgs<'a>),
    Return(UpdateReturn),
}
#[derive(Debug)]
pub enum UpdateArgs<'a> {
    Textured(crate::textured::UpdateArgs<'a>),
}
#[derive(Debug)]
pub enum UpdateReturn {
    Textured(crate::textured::UpdateReturn),
}

pub trait RenderConstruct {
    type Renderer: Renderer;
    type DrawParam;

    fn init_renderer(&mut self) -> Self::Renderer;
    fn draw(&mut self, rc: &mut RenderContext, record: &mut Record, data: Self::DrawParam);
}

pub trait Renderer {
    fn discriminant(&self) -> EntryDiscriminants;
    fn pre_render(&mut self, rc: &mut RenderContext, record: &Record, settings: &Settings);
    fn render(&mut self, rc: &mut RenderContext, rpass: &mut wgpu::RenderPass, entry: &Entry, settings: &Settings);
    fn post_render(&mut self, rc: &mut RenderContext, record: &Record, settings: &Settings);

    fn load<'a>(&mut self, rc: &mut RenderContext, update: Update<'a>) -> Update<'a>;
}

pub struct RenderRecordSystem {
    pub renderers: Vec<Box<dyn Renderer>>,
    pub renderer_mapping: HashMap<EntryDiscriminants, usize>,
}

pub struct Record {
    pub entries: Vec<Entry>,
}

impl Record {
    pub fn new() -> Self {
        Self {
            entries: vec![],
        }
    }
}

impl RenderRecordSystem {
    pub fn init() -> Self {
        Self {
            renderers: vec![],
            renderer_mapping: HashMap::new(),
        }
    }
    pub fn add<C>(&mut self, mut construct: C) -> C
            where C: RenderConstruct, C::Renderer: 'static {
        let r = construct.init_renderer();
        self.renderer_mapping.insert(r.discriminant(), self.renderers.len());
        let b: Box<dyn Renderer> = Box::new(r);
        self.renderers.push(b);
        return construct;
    }
    pub fn update<'a>(&mut self, rc: &mut RenderContext, target: &EntryDiscriminants, update: Update<'a>) -> Option<Update<'a>> {
        return self.renderer_mapping.get(target)
            .map(|index| &mut self.renderers[*index])
            .map(|renderer| renderer.load(rc, update));
    }
    pub fn render(&mut self, rc: &mut RenderContext, rpass: &mut wgpu::RenderPass<'_>, rr: &Record, settings: &Settings) {
        for renderer in &mut self.renderers {
            renderer.pre_render(rc, rr, settings);
        }
        for entry in &rr.entries {
            self.renderer_mapping.get_mut(&entry.into())
                .map(|index| &mut self.renderers[*index])
                .map(|renderer| renderer.render(rc, rpass, entry, settings));
        }
        for renderer in &mut self.renderers {
            renderer.post_render(rc, rr, settings);
        }
    }
}
