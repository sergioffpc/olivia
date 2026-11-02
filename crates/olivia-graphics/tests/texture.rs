//! Creating textures and regions, on a `Graphics` with no GPU.

use std::io::Cursor;

use image::{ImageFormat, Rgba, RgbaImage};
use olivia_graphics::{Config, Error, Graphics, Texture};
use olivia_math::{Rect, Vec2};

/// Four pixels, each with its own colour.
const PIXELS: [u8; 16] = [
    255, 0, 0, 255, 0, 255, 0, 255, // red, green
    0, 0, 255, 255, 255, 255, 255, 128, // blue, half-transparent white
];

fn graphics() -> Graphics {
    Graphics::new(Config {
        vsync: true,
        msaa: 4,
    })
}

fn texture() -> Texture {
    graphics().texture_rgba(2, 2, &PIXELS).unwrap()
}

/// A 2 by 1 image encoded as `format`.
fn encoded(format: ImageFormat) -> Vec<u8> {
    let image = RgbaImage::from_pixel(2, 1, Rgba([255, 0, 0, 255]));
    let mut bytes = Cursor::new(Vec::new());
    match format {
        // JPEG has no alpha channel.
        ImageFormat::Jpeg => image::DynamicImage::ImageRgba8(image)
            .to_rgb8()
            .write_to(&mut bytes, format),
        _ => image.write_to(&mut bytes, format),
    }
    .unwrap();
    bytes.into_inner()
}

#[test]
fn texture_rgba_keeps_its_pixels() {
    let texture = texture();
    assert_eq!(texture.size(), Vec2::new(2.0, 2.0));
    assert_eq!(texture.pixels(), PIXELS);
}

#[test]
fn texture_decodes_png_to_rgba() {
    let texture = graphics().texture(&encoded(ImageFormat::Png)).unwrap();
    assert_eq!(texture.size(), Vec2::new(2.0, 1.0));
    assert_eq!(texture.pixels(), [255, 0, 0, 255, 255, 0, 0, 255]);
}

#[test]
fn texture_decodes_jpeg() {
    let texture = graphics().texture(&encoded(ImageFormat::Jpeg)).unwrap();
    assert_eq!(texture.size(), Vec2::new(2.0, 1.0));
    assert_eq!(texture.pixels().len(), 8);
}

#[test]
fn undecodable_bytes_are_an_error() {
    let error = graphics().texture(b"not an image").unwrap_err();
    assert!(matches!(error, Error::UndecodableImage(_)));
}

#[test]
#[should_panic(expected = "4 bytes per pixel")]
fn too_few_rgba_bytes_panic() {
    let _ = graphics().texture_rgba(2, 2, &PIXELS[..12]);
}

#[test]
#[should_panic(expected = "4 bytes per pixel")]
fn sizes_that_overflow_panic() {
    let _ = graphics().texture_rgba(u32::MAX, u32::MAX, &PIXELS);
}

#[test]
#[should_panic(expected = "empty")]
fn an_empty_texture_panics() {
    let _ = graphics().texture_rgba(0, 4, &[]);
}

#[test]
fn a_clone_and_a_region_share_the_pixels() {
    let texture = texture();
    let region = texture.region(Rect::new(1.0, 0.0, 1.0, 2.0));
    assert_eq!(region.size(), Vec2::new(1.0, 2.0));
    assert!(std::ptr::eq(texture.pixels(), region.pixels()));
    assert!(std::ptr::eq(texture.pixels(), texture.clone().pixels()));
}

#[test]
fn a_region_may_cover_the_whole_texture() {
    let region = texture().region(Rect::new(0.0, 0.0, 2.0, 2.0));
    assert_eq!(region.size(), Vec2::new(2.0, 2.0));
}

#[test]
#[should_panic(expected = "outside the texture")]
fn a_region_past_the_edge_panics() {
    let _ = texture().region(Rect::new(1.0, 1.0, 2.0, 1.0));
}

#[test]
#[should_panic(expected = "outside the texture")]
fn a_region_before_the_origin_panics() {
    let _ = texture().region(Rect::new(-1.0, 0.0, 1.0, 1.0));
}

#[test]
#[should_panic(expected = "outside the texture")]
fn a_region_of_negative_size_panics() {
    let _ = texture().region(Rect::new(1.0, 1.0, -1.0, 1.0));
}

#[test]
#[should_panic(expected = "outside the texture")]
fn a_nan_region_panics() {
    let _ = texture().region(Rect::new(f32::NAN, 0.0, 1.0, 1.0));
}

#[test]
#[should_panic(expected = "outside the texture")]
fn a_region_is_bounded_by_the_region_it_is_taken_from() {
    let half = texture().region(Rect::new(1.0, 0.0, 1.0, 2.0));
    let _ = half.region(Rect::new(0.0, 0.0, 2.0, 1.0));
}
