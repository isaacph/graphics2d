use std::{borrow::Cow, str::from_utf8};
use crate::{rrs::{self, Entry, EntryDiscriminants, Record, RenderConstruct, Settings, Update}, win::RenderContext};

pub struct Construct {
    renderer: Option<Renderer>,
}

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
}

impl rrs::Renderer for Renderer {
    fn discriminant(&self) -> EntryDiscriminants {
        EntryDiscriminants::Simple
    }

    fn pre_render(&mut self, _rc: &mut RenderContext, _: &Record, _: &Settings) {
    }

    fn render(&mut self, _rc: &mut RenderContext, rpass: &mut wgpu::RenderPass, _entry: &Entry, _: &Settings) {
        rpass.set_pipeline(&self.pipeline);
        rpass.draw(0..3, 0..1);
    }

    fn post_render(&mut self, _rc: &mut RenderContext, _: &Record, _: &Settings) {
    }

    fn load<'a>(&mut self, _: &mut RenderContext, _: Update<'a>) -> Update<'a> {
        panic!("Update is a mistake for simple renderer");
    }
}
impl RenderConstruct for Construct {
    type DrawParam = ();
    type Renderer = Renderer;

    fn init_renderer(&mut self) -> Renderer {
        return self.renderer.take().expect("Cannot have multiuple renderers for a construct");
    }

    fn draw(&mut self, _rc: &mut RenderContext, record: &mut Record, _data: ()) {
        record.entries.push(Entry::Simple);
    }
}

impl Construct {
    pub fn init(rc: &mut RenderContext) -> Construct {
        let shader = rc.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                Cow::from(from_utf8(include_bytes!(env!("SIMPLE_SHADER"))).unwrap())),
        });
        let pipeline_layout = rc.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = rc.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(rc.surface_format.into())],
            }),
            multiview: None,
            cache: None,
        });
        return Construct {
            renderer: Some(Renderer {
                pipeline,
            })
        };
    }
}
