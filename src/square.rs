use wgpu::util::DeviceExt;

use crate::win::RenderContext;
use std::{borrow::Cow, str};

pub struct Square {
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
}

impl Square {
    pub fn init(rc: &mut RenderContext) -> Square {
        let shader = rc.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                Cow::from(str::from_utf8(include_bytes!(env!("SQUARE_SHADER"))).unwrap())),
        });
        let uniform_buffer = rc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &[],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_bind_group_layout = rc.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let uniform_bind_group = rc.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: todo!(),
                    }),
                },
            ],
        });
        let pipeline_layout = rc.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniform_bind_group_layout],
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
            uniform_bind_group,
            uniform_buffer,
        };
    }

    pub fn draw(&mut self, rc: &mut RenderContext, rpass: &mut wgpu::RenderPass) {
        rc.queue.write_buffer(&self.uniform_buffer, 0, todo!());

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }
}
