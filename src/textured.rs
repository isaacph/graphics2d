use std::collections::HashMap;

use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use crate::{camera::CameraObj, texture::Texture};

const INSTANCE_BUFFERS: u32 = 128;
const INSTANCE_BUFFER_DEFAULT: [InstanceRaw; 16] = [InstanceRaw { model: [[0.0; 4]; 4], color: [0.0; 4] }; 16];

use std::cell::Cell;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj<C: CameraObj>(&mut self, camera: &C) {
        self.view_proj = camera.proj_view().into();
        // self.view_proj = cgmath::Matrix4::identity().into();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 0.0], },
    Vertex { position: [-0.5,  0.5, 0.0], tex_coords: [0.0, 1.0], },
    Vertex { position: [ 0.5,  0.5, 0.0], tex_coords: [1.0, 1.0], },
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 0.0], },
    Vertex { position: [ 0.5,  0.5, 0.0], tex_coords: [1.0, 1.0], },
    Vertex { position: [ 0.5, -0.5, 0.0], tex_coords: [1.0, 0.0], },
];

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

#[derive(Clone)]
pub struct Instance {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    // pub rotation: f32,
    pub color: cgmath::Vector4<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        let translation = cgmath::Matrix4::from_translation(
            cgmath::Vector3::new(self.position.x, self.position.y, 0.0)
        );
        // let rotation = //cgmath::Matrix4::from_angle_z(cgmath::Rad(self.rotation));
        //             cgmath::Matrix4::identity();
        let scaling = cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 1.0);
        let transf = translation * scaling;
        InstanceRaw {
            model: transf.into(),
        //    model: cgmath::Matrix4::identity().into(),
            color: self.color.into(),
        }
    }
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in
                // the shader.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // for color
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct TextureBindGroups {
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: HashMap<String, wgpu::BindGroup>,
}

impl TextureBindGroups {
    pub fn new(texture_bind_group_layout: wgpu::BindGroupLayout) -> Self {
        Self {
            texture_bind_group_layout,
            texture_bind_groups: HashMap::new()
        }
    }

    pub fn make_texture_bind_group(&mut self, device: &wgpu::Device, texture: &Texture) {
        if !self.texture_bind_groups.contains_key(&texture.id) {
            // create bind group for texture
            let diffuse_bind_group = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &&self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                    ],
                    label: Some("diffuse_bind_group"),
                }
            );
            self.texture_bind_groups.insert(texture.id.clone(), diffuse_bind_group);
        }
    }

    pub fn get_texture_bind_group<'a>(&'a self, texture: &Texture) -> Option<&'a wgpu::BindGroup> {
        self.texture_bind_groups.get(&texture.id)
    }
}

pub struct TextureRenderer {
    render_pipeline: wgpu::RenderPipeline,

    square_vertex_buffer: wgpu::Buffer,
    square_num_vertices: u32,

    texture_bind_groups: TextureBindGroups,

    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,

    instance_buffer_list: Vec<wgpu::Buffer>,
    current_buffer: Cell<usize>,

    // color_bind_group: wgpu::BindGroup,
    // color_buffer: wgpu::Buffer,
}

impl TextureRenderer {
    pub fn add_texture<'a, T: Iterator>(&mut self, device: &wgpu::Device, textures: T) where T::Item : Into<&'a Texture> {
        textures.for_each(|texture| self.texture_bind_groups.make_texture_bind_group(device, texture.into()));
    }

    pub fn init(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        // create texture bind group layout, specifying how valid textures for this renderer are formatted
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // create default square vertex buffer
        let square_vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX
            }
        );
        let square_num_vertices = VERTICES.len() as u32;

        // create a camera: buffer + bind group
        // filler data before we specify a real camera later
        let camera_uniform_dummy = CameraUniform {
            view_proj: [[0.0; 4]; 4],
        };
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera_buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform_dummy]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &&camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        // create an instance buffer list
        let instance_buffer_list = (0..INSTANCE_BUFFERS).into_iter().map(|_| {
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("instance_buffer"),
                    contents: bytemuck::cast_slice(&INSTANCE_BUFFER_DEFAULT),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }
            )
        }).collect::<Vec<_>>();

        // create color buffer + bind group
        // let color_uniform_dummy = ColorRaw {
        //     color: [0.0; 4],
        // };
        // let color_buffer = device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some("color_buffer"),
        //         contents: bytemuck::cast_slice(&[color_uniform_dummy]),
        //         usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //     }
        // );
        // let color_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     entries: &[
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 3,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty : wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: None,
        //             },
        //             count: None,
        //         }
        //     ],
        //     label: Some("instance_bind_group_layout"),
        // });
        // let instance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &&instance_bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: instance_buffer.as_entire_binding(),
        //         }
        //     ],
        //     label: Some("instance_bind_group"),
        // });

        // create the render pipeline
        let render_pipeline = {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("texture_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("texture_shader.wgsl").into()),
            });

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                        &camera_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        Vertex::desc(),
                        InstanceRaw::desc()
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    //cull_mode: Some(wgpu::Face::Back),
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });
            render_pipeline
        };
        Self {
            render_pipeline,
            camera_buffer,
            camera_bind_group,
            texture_bind_groups: TextureBindGroups::new(texture_bind_group_layout),
            square_vertex_buffer,
            square_num_vertices,
            instance_buffer_list,
            current_buffer: Cell::new(0)
        }
    }

    pub fn reset(&self) {
        self.current_buffer.set(0)
    }

    pub fn render<'a, C: CameraObj>(&'a self, queue: &mut wgpu::Queue, render_pass: &mut wgpu::RenderPass<'a>, camera: &C, instance_pairs_input: Vec<(Vec<Instance>, &Texture)>) -> Result<(), wgpu::SurfaceError> {
        // update the camera
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

        // for (_, texture) in &instance_pairs_input {
        //     self.texture_bind_groups.make_texture_bind_group(device, texture);
        // }

        // split instances apart
        let mut instance_pairs = vec![];
        for (mut instances, texture) in instance_pairs_input {
            while !instances.is_empty() {
                instance_pairs.push(
                    (
                        instances.drain(0..std::cmp::min(instances.len(), INSTANCE_BUFFER_DEFAULT.len()))
                            .collect::<Vec<_>>(),
                        texture
                    )
                );
            }
        }
        let mut current_buffer = self.current_buffer.get();
        if self.instance_buffer_list.len() < instance_pairs.len() + current_buffer {
            panic!("Gave too many instances even after grouping! {} groups. Can only support {} number of groups of {} size that share the same texture",
                   instance_pairs.len(), INSTANCE_BUFFERS, INSTANCE_BUFFER_DEFAULT.len());
        }
        self.current_buffer.set(current_buffer + instance_pairs.len());
        for (instances, texture) in instance_pairs {
            // grab the next instance buffer and write the new data to it
            let instance_buffer = &self.instance_buffer_list[current_buffer];
            current_buffer += 1;
            let buffer = instances.iter().map(|instance| instance.to_raw()).collect::<Vec<_>>();
            let buffer_bytes = bytemuck::cast_slice(&buffer);
            queue.write_buffer(instance_buffer, 0, buffer_bytes);

            // retrieve bind group for the given texture
            let diffuse_bind_group = self.texture_bind_groups.get_texture_bind_group(texture)
                .expect("Could not find texture bind group, did you forget to register your texture?");

            // set up for drawing, shared state across instances
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.square_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

            render_pass.draw(0..self.square_num_vertices, 0..(instances.len() as u32));
        }
        Ok(())
    }
}
