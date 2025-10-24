use image::{flat::NormalForm, FlatSamples};
use wgpu::util::DeviceExt;

use crate::win::RenderContext;

#[derive(Debug)]
pub struct TextureInfo {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

pub fn assert_packed(samples: &FlatSamples<Vec<u8>>) {
    assert!(samples.is_normal(NormalForm::Unaliased));
    assert!(samples.is_normal(NormalForm::PixelPacked));
    assert!(samples.is_normal(NormalForm::ImagePacked));
    assert!(samples.is_normal(NormalForm::RowMajorPacked));
}

pub fn init_texture(
    rc: &mut RenderContext,
    png: &[u8],
    mag_filter: wgpu::FilterMode,
) -> Result<TextureInfo, image::ImageError> {
    let cursor = std::io::Cursor::new(png);
    let img = image::ImageReader::new(cursor).with_guessed_format()?.decode()?;
    let img_rgba8 = img.into_rgba8();
    let samples = img_rgba8.into_flat_samples();
    assert_packed(&samples);
    let slice = samples.as_slice();

    let texture = rc.device.create_texture_with_data(
        &rc.queue,
        &wgpu::TextureDescriptor {
            label: Some("image-rs texture"),
            size: wgpu::Extent3d {
                width: samples.layout.width,
                height: samples.layout.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING, // create_texture_with_data implicitly
            // adds COPY_DST though
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::MipMajor,
        slice,
    );
    let view = texture.create_view(&Default::default());
    let sampler = rc.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    return Ok(TextureInfo { texture, view, sampler });
}
