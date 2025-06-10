use image::{flat::NormalForm, FlatSamples};

pub fn assert_packed(samples: &FlatSamples<Vec<u8>>) {
    assert!(samples.is_normal(NormalForm::Unaliased));
    assert!(samples.is_normal(NormalForm::PixelPacked));
    assert!(samples.is_normal(NormalForm::ImagePacked));
    assert!(samples.is_normal(NormalForm::RowMajorPacked));
}
