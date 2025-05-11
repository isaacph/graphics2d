use std::collections::HashMap;

use strum::EnumDiscriminants;

use crate::win::RenderContext;

pub trait RenderConstruct<D, R: Renderer> {
    fn init_renderer(&mut self) -> R;
    fn draw(&mut self, rc: &mut RenderContext, record: &mut RenderRecord, data: D);
}

pub trait Renderer {
    fn discriminant(&self) -> RenderRecordEntryDiscriminants;
    fn pre_render(&mut self, rc: &mut RenderContext, record: &RenderRecord);
    fn render(&mut self, rc: &mut RenderContext, rpass: &mut wgpu::RenderPass, entry: &RenderRecordEntry);
    fn post_render(&mut self, rc: &mut RenderContext, record: &RenderRecord);
}

pub struct RenderRecordSystem {
    pub renderers: Vec<Box<dyn Renderer>>,
    pub renderer_mapping: HashMap<RenderRecordEntryDiscriminants, usize>,
}

pub struct RenderRecord {
    pub entries: Vec<RenderRecordEntry>,
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
pub enum RenderRecordEntry {
    Simple,
    Square(crate::square::SquareRenderParams),
}

// what I need to make it work:
// draw methods that store state into RRE and into render type
// pre-render method called for each render type
// render method handing RRE to the correct types
// post-render method called for each render type

// dynamic dispatch per rendered object
// draw logic is separate from render logic
// keep an enum up to date with each new renderer

impl RenderRecord {
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
    pub fn add<D, R: Renderer + 'static, T: RenderConstruct<D, R>>(&mut self, mut construct: T) -> T {
        let r = construct.init_renderer();
        self.renderer_mapping.insert(r.discriminant(), self.renderers.len());
        let b: Box<dyn Renderer> = Box::new(r);
        self.renderers.push(b);
        return construct;
    }
    pub fn render(&mut self, rc: &mut RenderContext, rpass: &mut wgpu::RenderPass<'_>, rr: &RenderRecord) {
        for renderer in &mut self.renderers {
            renderer.pre_render(rc, rr);
        }
        for entry in &rr.entries {
            self.renderer_mapping.get_mut(&entry.into())
                .map(|index| &mut self.renderers[*index])
                .map(|renderer| renderer.render(rc, rpass, entry));
        }
        for renderer in &mut self.renderers {
            renderer.post_render(rc, rr);
        }
    }
}

