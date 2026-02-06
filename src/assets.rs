pub struct RasterImage {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u8>,
}

pub fn load_png_rgba(path: &str) -> RasterImage {
    let img = image::open(path).expect("Failed to open PNG");
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    RasterImage {
        width: width.try_into().expect("PNG width exceeds u16"),
        height: height.try_into().expect("PNG height exceeds u16"),
        pixels: rgba.into_raw(),
    }
}

pub fn scale_to_fit(raw_w: f32, raw_h: f32, max_size: f32) -> (f32, f32) {
    let scale = (max_size / raw_w).min(max_size / raw_h).min(1.0);
    (raw_w * scale, raw_h * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_to_fit_preserves_aspect_ratio() {
        let (w, h) = scale_to_fit(400.0, 200.0, 200.0);
        assert!((w - 200.0).abs() < f32::EPSILON);
        assert!((h - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn load_png_rgba_reads_pixels() {
        let png_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/png/signifiersmark.png");
        let raster = load_png_rgba(png_path.to_str().expect("Invalid path"));

        assert!(raster.width > 0);
        assert!(raster.height > 0);
        assert_eq!(
            raster.pixels.len(),
            raster.width as usize * raster.height as usize * 4
        );
    }

    #[test]
    fn signifier_mark_has_transparency() {
        let png_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/png/signifiersmark.png");
        let raster = load_png_rgba(png_path.to_str().expect("Invalid path"));

        let mut has_transparent = false;
        let mut has_opaque = false;

        for pixel in raster.pixels.chunks(4) {
            let alpha = pixel[3];
            if alpha == 0 {
                has_transparent = true;
            } else {
                has_opaque = true;
            }

            if has_transparent && has_opaque {
                break;
            }
        }

        assert!(
            has_transparent,
            "expected transparent pixels in signifier mark"
        );
        assert!(has_opaque, "expected opaque pixels in signifier mark");
    }
}
