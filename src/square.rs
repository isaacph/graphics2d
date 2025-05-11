use wgpu::util::DeviceExt;

use crate::{mat::{vec2, Mat4}, win::RenderContext};
use std::{borrow::Cow, num::NonZero, str};

pub struct Square {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
}

impl Square {
    pub fn init(rc: &mut RenderContext) -> Square {
        let bind_group_layout = rc.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZero::new(std::mem::size_of::<Mat4>() as u64).unwrap()),
                },
                count: None,
            }],
        });
        let uniform_buf = rc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: Mat4::identity().as_ref(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = rc.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let shader = rc.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                Cow::from(str::from_utf8(include_bytes!(env!("SQUARE_SHADER"))).unwrap())),
        });
        let pipeline_layout = rc.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
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
        return Square {
            pipeline,
            bind_group,
            uniform_buf,
        };
    }

    pub fn draw(&mut self, rc: &mut RenderContext, rpass: &mut wgpu::RenderPass, matrix: &Mat4) {
        rc.queue.write_buffer(&self.uniform_buf, 0, matrix.as_ref());
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }
}
