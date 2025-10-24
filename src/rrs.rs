use r#abstract::RenderRecord;
use strum::EnumDiscriminants;

use crate::{mat::Mat4, rrs::r#abstract::RenderRecordSystem};

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
pub enum Update {
    Empty,
    Args(UpdateArgs),
    Return(UpdateReturn),
}
#[derive(Debug)]
pub enum UpdateArgs {
    Textured(crate::textured::UpdateArgs),
}
#[derive(Debug)]
pub enum UpdateReturn {
    Textured(crate::textured::UpdateReturn),
}

pub type RecordSystem = RenderRecordSystem<Entry, EntryDiscriminants, Settings, Update>;
pub type Record = RenderRecord<Entry>;

pub mod r#abstract {
    use std::collections::HashMap;

    use crate::win::RenderContext;

    pub trait RenderConstruct<E, D, S> where E: 'static, D: std::hash::Hash + Eq + From<&'static E> {
        type Update;
        type Renderer: Renderer<E, D, Settings = S, Update = Self::Update>;
        type DrawParam;

        fn init_renderer(&mut self) -> Self::Renderer;
        fn draw(&mut self, rc: &mut RenderContext, record: &mut RenderRecord<E>, data: Self::DrawParam);
    }

    pub trait Renderer<E, D> where E: 'static, D: std::hash::Hash + Eq + From<&'static E> {
        type Settings;
        type Update;

        fn discriminant(&self) -> D;
        fn pre_render(&mut self, rc: &mut RenderContext, record: &RenderRecord<E>, settings: &Self::Settings);
        fn render(&mut self, rc: &mut RenderContext, rpass: &mut wgpu::RenderPass, entry: &E, settings: &Self::Settings);
        fn post_render(&mut self, rc: &mut RenderContext, record: &RenderRecord<E>, settings: &Self::Settings);

        fn load(&mut self, rc: &mut RenderContext, update: Self::Update) -> Self::Update;
    }

    pub struct RenderRecordSystem<E, D, S, U> where E: 'static, D: std::hash::Hash + Eq + From<&'static E> {
        pub renderers: Vec<Box<dyn Renderer<E, D, Settings = S, Update = U>>>,
        pub renderer_mapping: HashMap<D, usize>,
    }

    pub struct RenderRecord<E> {
        pub entries: Vec<E>,
    }

    impl<E> RenderRecord<E> {
        pub fn new() -> Self {
            Self {
                entries: vec![],
            }
        }
    }

    impl<E, D, S, U> RenderRecordSystem<E, D, S, U> where D: std::hash::Hash + Eq + for<'a> From<&'a E> {
        pub fn init() -> Self {
            Self {
                renderers: vec![],
                renderer_mapping: HashMap::new(),
            }
        }
        pub fn add<C>(&mut self, mut construct: C) -> C
                where C: RenderConstruct<E, D, S, Update = U>, C::Renderer: 'static {
            let r = construct.init_renderer();
            self.renderer_mapping.insert(r.discriminant(), self.renderers.len());
            let b: Box<dyn Renderer<E, D, Settings = S, Update = U>> = Box::new(r);
            self.renderers.push(b);
            return construct;
        }
        pub fn update(&mut self, rc: &mut RenderContext, target: &D, update: U) -> Option<U> {
            return self.renderer_mapping.get(target)
                .map(|index| &mut self.renderers[*index])
                .map(|renderer| renderer.load(rc, update));
        }
        pub fn render(&mut self, rc: &mut RenderContext, rpass: &mut wgpu::RenderPass<'_>, rr: &RenderRecord<E>, settings: &S) {
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
}


