use image::GenericImageView;
use anyhow::*;

pub struct Texture {
    pub id: String,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn blank_texture(device: &wgpu::Device, queue: &wgpu::Queue, id: &str) -> Result<Self> {
        let mut image = image::DynamicImage::new_rgba8(1, 1);
        if let Some(pixels) = image.as_mut_rgba8() {
            for x in 0..pixels.len() {
                if let Some(pixel) = pixels.get_mut(x) {
                    *pixel = 255;
                }
            }
        }
        Self::from_image(device, queue, &image, id, wgpu::FilterMode::Nearest)
    }

    pub fn from_image_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        id: &str,
        filter: wgpu::FilterMode
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, id, filter)
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        diffuse: &[u8],
        dimensions: (u32, u32),
        id: &str,
        format: wgpu::TextureFormat,
        bytes_per_pixel: u32,
        dimension: wgpu::TextureDimension,
        mag_filter: wgpu::FilterMode,
    ) -> Result<Self> {
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some((String::from("diffuse_texture_") + id).as_str()),
                // All textures are stored as 3D, we represent our 2D
                // texture by setting depth to 1
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension,
                // reflecting that most images are stored as sRGB
                format,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            }
        );
        // load the texture into with the queue
        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // the actual pixel data
            &diffuse,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(bytes_per_pixel * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            sampler,
            view,
            id: String::from(id),
        })
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        id: &str,
        filter: wgpu::FilterMode,
    ) -> Result<Self> {
        let diffuse_rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        Self::from_bytes(
            device,
            queue,
            &diffuse_rgba,
            dimensions,
            id,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            4,
            wgpu::TextureDimension::D2,
            filter
        )
    }
}
