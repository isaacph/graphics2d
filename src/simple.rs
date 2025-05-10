use crate::win::RenderContext;
use std::{borrow::Cow, str};

pub struct Simple {
    pipeline: wgpu::RenderPipeline,
}

impl Simple {
    pub fn init(rc: &mut RenderContext) -> Simple {
        let shader = rc.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                Cow::from(str::from_utf8(include_bytes!(env!("SIMPLE_SHADER"))).unwrap())),
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
        return Simple {
            pipeline,
        };
    }

    pub fn draw(&mut self, _rc: &mut RenderContext, rpass: &mut wgpu::RenderPass) {
        rpass.set_pipeline(&self.pipeline);
        rpass.draw(0..3, 0..1);
    }
}
