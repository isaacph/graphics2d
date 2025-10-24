use crate::{
    mat::Mat4,
    rrs::{self, Entry, EntryDiscriminants, Record, RenderConstruct, RenderRecordSystem, Settings, Update},
    texture::TextureInfo,
    util::indirect_handles::{Handle, HandleTracker, WeakHandle},
    win::RenderContext,
};
use std::{borrow::Cow, num::NonZero, str};
use wgpu::{self, util::DeviceExt};

pub struct Construct(Option<Renderer>);

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Texture;

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    instance_buf_count: usize,
    current_buf: u32,
    bind_groups: HandleTracker<Texture, TextureBindGroup>,
}

#[derive(Debug)]
pub struct RenderParams {
    pub matrix: Mat4,
    pub texture: WeakHandle<Texture>,
}

#[derive(Debug)]
pub struct UpdateArgs<'a>(&'a TextureInfo);
#[derive(Debug)]
pub struct UpdateReturn(Handle<Texture>);

#[derive(Debug)]
struct TextureBindGroup {
    bind_group: wgpu::BindGroup,
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
        let bind_group_layout =
            rc.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                NonZero::new(std::mem::size_of::<Mat4>() as u64).unwrap(),
                            ),
                        },
                        count: None,
                    }, wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    }, wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    }],
                });
        let uniform_buf = rc
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: Mat4::identity().as_ref(),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });
        let instance_buf = rc
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &[],
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            });
        let instance_buf_count = 0;

        let shader = rc
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::from(
                    str::from_utf8(include_bytes!(env!("TEXTURE_SHADER"))).unwrap(),
                )),
            });
        let pipeline_layout = rc
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline = rc
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            bind_group_layout,
            uniform_buf,
            instance_buf_count,
            instance_buf,
            current_buf: 0,
            bind_groups: Default::default(),
        }));
    }
}

impl RenderConstruct for Construct {
    type Renderer = Renderer;
    type DrawParam = RenderParams;

    fn init_renderer(&mut self) -> Renderer {
        self.0
            .take()
            .expect("Cannot instantiate multiple renderers for a construct")
    }

    fn draw(&mut self, _rc: &mut RenderContext, record: &mut Record, data: RenderParams) {
        record.entries.push(Entry::Textured(data));
    }
}

impl rrs::Renderer for Renderer {
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
            self.instance_buf = rc
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &vec![0; new_count * size_of::<InstanceBuffer>()].into_boxed_slice(),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                });
            self.instance_buf_count = new_count;
        }
        self.current_buf = 0;
    }

    fn render(
        &mut self,
        rc: &mut RenderContext,
        rpass: &mut wgpu::RenderPass,
        entry: &Entry,
        _: &Settings,
    ) {
        let RenderParams { matrix, texture } = match entry {
            Entry::Textured(p) => p,
            _ => panic!("Failed to call correct renderer!"),
        };
        let TextureBindGroup { bind_group } = self.bind_groups.get(texture)
            .expect("TODO: add default textures when dropped textures");

        let buf_index = self.current_buf;
        self.current_buf += 1;
        let offset: u64 = (size_of::<InstanceBuffer>() * (buf_index as usize))
            .try_into()
            .unwrap();
        rc.queue
            .write_buffer(&self.instance_buf, offset, matrix.as_ref());
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, Some(bind_group), &[]);
        rpass.set_vertex_buffer(0, self.instance_buf.slice(..));
        rpass.draw(0..6, buf_index..buf_index + 1);
    }

    fn post_render(&mut self, rc: &mut RenderContext, _: &Record, settings: &Settings) {
        self.current_buf = 0;
        rc.queue.write_buffer(&self.uniform_buf, 0, settings.projection.as_ref().into());
    }

    fn load<'a>(&mut self, rc: &mut RenderContext, update: Update<'a>) -> Update<'a> {
        match update {
            Update::Args(crate::rrs::UpdateArgs::Textured(UpdateArgs(
                TextureInfo {
                    texture: _,
                    view,
                    sampler,
                }
            ))) => {
                let bind_group = rc.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buf.as_entire_binding(),
                    }, wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    }, wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }],
                });
                let bind_group = TextureBindGroup {
                    bind_group,
                };
                let handle = self.bind_groups.put(bind_group);
                return Update::Return(crate::rrs::UpdateReturn::Textured(UpdateReturn(handle)));
            }
            _ => panic!("Invalid update for texture renderer: {:?}", update),
        }
    }
}

impl Construct {
    pub fn init_texture<'a>(
        &mut self,
        rc: &mut RenderContext,
        rrs: &mut RenderRecordSystem,
        texture_info: &'a TextureInfo,
    ) -> Handle<Texture> {
        // TODO: insane boilerplate to store the result in the renderer
        // maybe I should replace this with reference counted refcell?
        let args = Update::Args(crate::rrs::UpdateArgs::Textured(UpdateArgs(texture_info)));
        match rrs.update(rc, &EntryDiscriminants::Textured, args).expect("Texture update returned None") {
            Update::Return(crate::rrs::UpdateReturn::Textured(update_return)) => update_return.0,
            other_update => panic!("Invalid update returned for texture renderer: {:?}", other_update),
        }
    }
}
