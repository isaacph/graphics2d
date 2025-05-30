use wgpu::util::DeviceExt;

use crate::{mat::Mat4, rrs::{r#abstract::RenderConstruct, Entry, EntryDiscriminants, Record, Settings}, win::RenderContext};
use std::{borrow::Cow, num::NonZero, str};

pub struct Construct(Option<Renderer>);

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    instance_buf_count: usize,
    current_buf: u32,
}

#[derive(Debug)]
pub struct RenderParams {
    pub matrix: Mat4,
}

#[repr(C, packed)]
struct InstanceBuffer {
    matrix: Mat4,
}

impl InstanceBuffer {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x4,
            1 => Float32x4,
            2 => Float32x4,
            3 => Float32x4,
        ],
    };
}

impl Construct {
    pub fn init(rc: &mut RenderContext) -> Construct {
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

        let instance_buf = rc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &[],
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });
        let instance_buf_count = 0;

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
                buffers: &[InstanceBuffer::LAYOUT],
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
        return Construct(Some(Renderer {
            pipeline,
            bind_group,
            uniform_buf,
            instance_buf_count,
            instance_buf,
            current_buf: 0,
        }));
    }
}

impl RenderConstruct<Entry, EntryDiscriminants, Settings> for Construct {
    type Renderer = Renderer;
    type DrawParam = RenderParams;

    fn init_renderer(&mut self) -> Renderer {
        self.0.take().expect("Cannot instantiate multiple renderers for a construct")
    }

    fn draw(&mut self, _rc: &mut RenderContext, record: &mut Record, data: RenderParams) {
        record.entries.push(Entry::Textured(data));
    }
}

impl crate::rrs::r#abstract::Renderer<Entry, EntryDiscriminants> for Renderer {
    type Settings = Settings;

    fn discriminant(&self) -> EntryDiscriminants {
        EntryDiscriminants::Textured
    }

    fn pre_render(&mut self, rc: &mut RenderContext, record: &Record, _: &Settings) {
        let mut new_count: usize = 0;
        for entry in &record.entries {
            match entry {
                Entry::Textured(_) => new_count += 1,
                _ => (),
            }
        }
        if new_count > self.instance_buf_count {
            self.instance_buf = rc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &vec![0; new_count * size_of::<InstanceBuffer>()].into_boxed_slice(),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            });
            self.instance_buf_count = new_count;
        }
        self.current_buf = 0;
    }

    fn render(&mut self, rc: &mut RenderContext, rpass: &mut wgpu::RenderPass, entry: &Entry, _: &Settings) {
        let RenderParams {
            matrix,
        } = match entry {
            Entry::Textured(p) => p,
            _ => panic!("Failed to call correct renderer!"),
        };

        let buf_index = self.current_buf;
        self.current_buf += 1;
        let offset: u64 = (size_of::<InstanceBuffer>() * (buf_index as usize)).try_into().unwrap();
        rc.queue.write_buffer(&self.instance_buf, offset, matrix.as_ref());

        rc.queue.write_buffer(&self.uniform_buf, 0, Mat4::identity().as_ref());
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.instance_buf.slice(..));
        rpass.draw(0..6, buf_index..buf_index+1);
    }

    fn post_render(&mut self, _rc: &mut RenderContext, _: &Record, _: &Settings) {
        self.current_buf = 0;
    }
}


