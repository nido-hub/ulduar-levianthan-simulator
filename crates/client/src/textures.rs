//! Original painted-style texture recipes. No game-client images are used.
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Surface {
    Wood,
    Iron,
    Bronze,
    Stone,
    Trim,
    Leather,
    Cloth,
    Horn,
    Floor,
}

pub struct Paint {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}
impl Paint {
    fn new(size: u32, color: [u8; 3]) -> Self {
        Self {
            width: size,
            height: size,
            pixels: (0..size * size)
                .flat_map(|_| [color[0], color[1], color[2], 255])
                .collect(),
        }
    }
    fn blend(&mut self, x: i32, y: i32, color: [u8; 3], alpha: f32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let i = (y as usize * self.width as usize + x as usize) * 4;
        for channel in 0..3 {
            self.pixels[i + channel] = (self.pixels[i + channel] as f32 * (1.0 - alpha)
                + color[channel] as f32 * alpha)
                .clamp(0.0, 255.0) as u8;
        }
    }
    fn line(&mut self, a: (f32, f32), b: (f32, f32), width: f32, color: [u8; 3], alpha: f32) {
        let steps = ((b.0 - a.0).hypot(b.1 - a.1) * 1.4).ceil().max(1.0) as i32;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = a.0 + (b.0 - a.0) * t;
            let y = a.1 + (b.1 - a.1) * t;
            let radius = width.ceil() as i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let d = (dx as f32).hypot(dy as f32);
                    if d <= width {
                        self.blend(
                            x as i32 + dx,
                            y as i32 + dy,
                            color,
                            alpha * (width - d).min(1.0),
                        );
                    }
                }
            }
        }
    }
    fn circle(&mut self, x: f32, y: f32, radius: f32, color: [u8; 3], alpha: f32) {
        for dy in -(radius as i32 + 1)..=radius as i32 + 1 {
            for dx in -(radius as i32 + 1)..=radius as i32 + 1 {
                let d = (dx as f32).hypot(dy as f32) / radius;
                if d < 1.0 {
                    self.blend(
                        x as i32 + dx,
                        y as i32 + dy,
                        color,
                        alpha * (1.0 - d).min(0.6),
                    );
                }
            }
        }
    }
}
fn hash(x: u32, y: u32, seed: u32) -> u32 {
    let mut value = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ seed;
    value = (value ^ (value >> 13)).wrapping_mul(1274126177);
    value ^ (value >> 16)
}
fn noise(x: f32, y: f32, scale: f32, seed: u32) -> f32 {
    let x = x / scale;
    let y = y / scale;
    let ix = x.floor() as u32;
    let iy = y.floor() as u32;
    let fx = x.fract();
    let fy = y.fract();
    let fx = fx * fx * (3.0 - 2.0 * fx);
    let fy = fy * fy * (3.0 - 2.0 * fy);
    let n = |a, b| (hash(a, b, seed) & 65535) as f32 / 65535.0 - 0.5;
    let a = n(ix, iy) * (1.0 - fx) + n(ix + 1, iy) * fx;
    let b = n(ix, iy + 1) * (1.0 - fx) + n(ix + 1, iy + 1) * fx;
    a * (1.0 - fy) + b * fy
}
pub fn recipe(surface: Surface) -> Paint {
    let size = if surface == Surface::Floor { 2048 } else { 512 };
    let color = match surface {
        Surface::Wood => [132, 91, 43],
        Surface::Iron => [104, 113, 100],
        Surface::Bronze => [154, 125, 69],
        Surface::Stone | Surface::Floor => [74, 66, 80],
        Surface::Trim => [134, 122, 139],
        Surface::Leather => [97, 57, 34],
        Surface::Cloth => [139, 48, 37],
        Surface::Horn => [191, 171, 125],
    };
    let mut paint = Paint::new(size, color);
    for y in 0..size {
        for x in 0..size {
            let xf = x as f32;
            let yf = y as f32;
            let broad = noise(xf, yf, 85.0, 17) * 20.0 + noise(xf, yf, 22.0, 53) * 8.0;
            let fine = ((hash(x, y, 74) & 255) as f32 / 255.0 - 0.5) * 4.0;
            let edge = xf
                .min(yf)
                .min((size - 1 - x) as f32)
                .min((size - 1 - y) as f32);
            let shade = match surface {
                Surface::Wood => {
                    let plank = (x / 128) as f32;
                    let grain =
                        (xf * 0.21 + noise(xf, yf, 50.0, 41) * 3.0 + (yf * 0.025).sin()).sin();
                    let seam = x % 128;
                    broad + grain * 10.0 + noise(xf, yf * 0.08, 8.0, 21) * 18.0 + plank * 3.0
                        - if seam < 4 { 55.0 } else { 0.0 }
                        + if (5..9).contains(&seam) { 20.0 } else { 0.0 }
                }
                Surface::Stone => {
                    let row = y / 128;
                    let xx = (x + (row % 2) * 128) % 256;
                    let yy = y % 128;
                    let e = xx.min(255 - xx).min(yy).min(127 - yy) as f32;
                    broad - (1.0 - e / 12.0).max(0.0) * 30.0
                        + if (4.0..7.0).contains(&e) { 13.0 } else { 0.0 }
                }
                Surface::Floor => {
                    let row = y / 128;
                    let xx = (x + row % 2 * 64) % 128;
                    let yy = y % 128;
                    let e = xx.min(127 - xx).min(yy).min(127 - yy) as f32;
                    let slab = (hash((x + row % 2 * 64) / 128, row, 812) % 13) as f32 - 6.0;
                    broad + slab - (1.0 - e / 10.0).max(0.0) * 34.0
                        + if (3.0..6.0).contains(&e) { 17.0 } else { 0.0 }
                }
                Surface::Iron | Surface::Bronze => {
                    broad - (1.0 - edge / 32.0).max(0.0) * 28.0
                        + if (9.0..15.0).contains(&edge) {
                            28.0
                        } else {
                            0.0
                        }
                }
                Surface::Trim => {
                    broad + (yf / 40.0).sin() * 5.0 - (1.0 - edge / 24.0).max(0.0) * 25.0
                }
                Surface::Leather | Surface::Cloth => {
                    broad + if (x + y) % 4 == 0 { 3.0 } else { -1.0 }
                        - (1.0 - edge / 35.0).max(0.0) * 20.0
                }
                Surface::Horn => broad + (xf * 0.05 + (yf * 0.015).sin()).sin() * 9.0,
            };
            let i = ((y * size + x) * 4) as usize;
            for channel in 0..3 {
                paint.pixels[i + channel] =
                    (color[channel] as f32 + shade + fine).clamp(0.0, 255.0) as u8;
            }
        }
    }
    match surface {
        Surface::Wood => {
            for i in 0..100 {
                let x = (hash(i, 5, 8) % 512) as f32;
                let y = (hash(i, 2, 9) % 512) as f32;
                let len = 15.0 + (hash(i, 8, 4) % 80) as f32;
                paint.line((x, y), (x + 2.5, y + len), 1.4, [57, 39, 21], 0.4);
                paint.line((x + 2.0, y), (x + 4.0, y + len), 0.8, [217, 168, 91], 0.45);
            }
            for (x, y) in [(72.0, 165.0), (330.0, 380.0), (200.0, 85.0)] {
                for ring in [9.0_f32, 15.0, 23.0] {
                    for j in 0..36 {
                        let a = j as f32 * std::f32::consts::TAU / 36.0;
                        let b = a + 0.18;
                        paint.line(
                            (x + ring * 0.4 * a.cos(), y + ring * a.sin()),
                            (x + ring * 0.4 * b.cos(), y + ring * b.sin()),
                            1.1,
                            [69, 42, 21],
                            0.5,
                        );
                    }
                }
            }
        }
        Surface::Iron | Surface::Bronze => {
            for i in 0..40 {
                let x = (hash(i, 1, 56) % 480 + 16) as f32;
                let y = (hash(i, 2, 24) % 480 + 16) as f32;
                paint.circle(
                    x,
                    y,
                    7.0 + (hash(i, 3, 3) % 14) as f32,
                    if surface == Surface::Bronze {
                        [72, 109, 96]
                    } else {
                        [126, 77, 36]
                    },
                    0.27,
                );
                paint.line((x, y), (x + 10.0, y - 16.0), 0.9, [199, 192, 148], 0.55);
            }
            for x in [29.0, 483.0] {
                for y in [30.0, 256.0, 482.0] {
                    paint.circle(x, y + 3.0, 12.0, [21, 28, 21], 1.0);
                    paint.circle(x, y, 9.0, [194, 191, 147], 1.0);
                    paint.line(
                        (x - 3.0, y - 2.0),
                        (x + 3.0, y - 2.0),
                        1.3,
                        [224, 215, 177],
                        0.8,
                    );
                }
            }
        }
        Surface::Floor | Surface::Stone => {
            for i in 0..if surface == Surface::Floor { 100 } else { 35 } {
                let x = (hash(i, 2, 55) % size) as f32;
                let y = (hash(i, 8, 19) % size) as f32;
                let bend = (x + 8.0, y + 11.0);
                let end = (x + 16.0, y + 26.0);
                paint.line((x, y), bend, 1.3, [32, 30, 41], 0.65);
                paint.line(bend, end, 1.0, [32, 30, 41], 0.65);
                paint.line(
                    (x + 2.0, y),
                    (bend.0 + 2.0, bend.1),
                    0.8,
                    [149, 137, 154],
                    0.35,
                );
            }
            if surface == Surface::Floor {
                let size = size as f32;
                // Low-contrast inlaid borders, not noisy photographic rock.
                for edge in [175.0, size - 175.0] {
                    paint.line((edge, 175.0), (edge, size - 175.0), 8.0, [46, 40, 53], 0.85);
                    paint.line(
                        (edge + 10.0, 175.0),
                        (edge + 10.0, size - 175.0),
                        2.0,
                        [139, 122, 119],
                        0.6,
                    );
                    paint.line((175.0, edge), (size - 175.0, edge), 8.0, [46, 40, 53], 0.85);
                    paint.line(
                        (175.0, edge + 10.0),
                        (size - 175.0, edge + 10.0),
                        2.0,
                        [139, 122, 119],
                        0.6,
                    );
                }
                for i in 0..90 {
                    let x = (hash(i, 7, 64) % size as u32) as f32;
                    let y = (hash(i, 9, 91) % size as u32) as f32;
                    paint.circle(
                        x,
                        y,
                        22.0 + (hash(i, 3, 45) % 40) as f32,
                        [38, 33, 43],
                        0.14,
                    );
                }
            }
        }
        Surface::Trim => {
            for i in 0..8 {
                let y = i as f32 * 64.0;
                for x in [85.0, 341.0] {
                    for (a, b) in [
                        ((x, y + 12.0), (x + 45.0, y + 12.0)),
                        ((x + 45.0, y + 12.0), (x + 45.0, y + 48.0)),
                        ((x + 45.0, y + 48.0), (x + 14.0, y + 48.0)),
                        ((x + 14.0, y + 48.0), (x + 14.0, y + 28.0)),
                        ((x + 14.0, y + 28.0), (x + 31.0, y + 28.0)),
                    ] {
                        paint.line(a, b, 4.0, [61, 51, 69], 0.8);
                        paint.line(
                            (a.0 - 2.0, a.1 - 2.0),
                            (b.0 - 2.0, b.1 - 2.0),
                            1.3,
                            [202, 181, 186],
                            0.65,
                        );
                    }
                }
            }
        }
        Surface::Leather | Surface::Cloth => {
            for y in (12..500).step_by(14) {
                for x in [14, 498] {
                    paint.line(
                        (x as f32, y as f32),
                        (x as f32, y as f32 + 6.0),
                        1.1,
                        [201, 164, 103],
                        0.75,
                    );
                }
            }
        }
        _ => {}
    }
    paint
}

pub fn image(surface: Surface) -> Image {
    let paint = recipe(surface);
    let mut image = Image::new(
        Extent3d {
            width: paint.width,
            height: paint.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        paint.pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: 8,
        ..default()
    });
    image
}

/// Small original flame card for native combat effects (transparent, not a solid primitive).
pub fn flame_image() -> Image {
    let (width, height) = (128, 256);
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let h = 1. - y as f32 / (height - 1) as f32;
            let u = x as f32 / (width - 1) as f32 * 2. - 1.;
            let center = (h * 9.).sin() * 0.12 + h * 0.08;
            let half = (1. - h).powf(0.8) * (0.64 + (h * 13.).sin() * 0.07);
            let body = ((half - (u - center).abs()) * 12.).clamp(0., 1.);
            let tongue = if h < 0.7 {
                ((0.23 * (1. - h / 0.7) - ((u + 0.45) - (h * 8.).sin() * 0.04).abs()) * 15.)
                    .clamp(0., 1.)
            } else {
                0.
            };
            let alpha = (body.max(tongue) * (h * 18.).clamp(0., 1.) * 255.) as u8;
            let inner = if half > 0.001 {
                (1. - (u - center).abs() / half).clamp(0., 1.)
            } else {
                0.
            };
            pixels.extend_from_slice(&[
                255,
                (95. + 135. * inner * (1. - h * 0.5)) as u8,
                (12. + 145. * inner * inner * (1. - h)) as u8,
                alpha,
            ]);
        }
    }
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Explicit material assignment for the first authored scene, not a global hue heuristic.
pub fn surface(hex: u32) -> Option<Surface> {
    Some(match hex {
        0x6f5531 | 0x927443 | 0xb1945d | 0x9d7e4c | 0x78572f | 0xae8a4c | 0x87693c => Surface::Wood,
        0x343934 | 0x777d68 | 0x707969 | 0x7e8375 | 0x706b4c | 0x8c9480 | 0x66705d | 0x929b9d => {
            Surface::Iron
        }
        0xa9ad95 | 0xbea571 | 0xddddba | 0xb8b395 | 0x797c78 | 0x878774 => Surface::Bronze,
        0xcfc49d | 0xd3c59e => Surface::Horn,
        0x885540 => Surface::Leather,
        0x903b31 => Surface::Cloth,
        0x5e596e | 0x736c80 | 0x716b81 | 0x746b81 => Surface::Stone,
        0x9690a4 | 0x655d71 | 0x6b6277 | 0x93879d | 0x9c93a6 | 0xa69aae => Surface::Trim,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flame_has_transparent_edges_and_opaque_core() {
        let image = flame_image();
        let bytes = image.data.unwrap();
        assert!(bytes.chunks_exact(4).any(|p| p[3] == 0));
        assert!(bytes.chunks_exact(4).any(|p| p[3] > 240));
    }
    #[test]
    fn all_textures_are_opaque_nonuniform_and_correctly_sized() {
        for kind in [
            Surface::Wood,
            Surface::Iron,
            Surface::Bronze,
            Surface::Stone,
            Surface::Trim,
            Surface::Leather,
            Surface::Cloth,
            Surface::Horn,
            Surface::Floor,
        ] {
            let paint = recipe(kind);
            assert_eq!(
                paint.pixels.len(),
                (paint.width * paint.height * 4) as usize
            );
            assert!(paint.pixels.chunks_exact(4).all(|p| p[3] == 255));
            let min = paint.pixels.chunks_exact(4).map(|p| p[0]).min().unwrap();
            let max = paint.pixels.chunks_exact(4).map(|p| p[0]).max().unwrap();
            assert!(max - min > 25, "{kind:?}");
        }
    }
    #[test]
    fn recipes_are_reproducible() {
        assert_eq!(recipe(Surface::Iron).pixels, recipe(Surface::Iron).pixels);
    }
    #[test]
    fn wooden_frame_and_metal_are_separate() {
        assert_eq!(surface(0xae8a4c), Some(Surface::Wood));
        assert_eq!(surface(0x777d68), Some(Surface::Iron));
    }
}
