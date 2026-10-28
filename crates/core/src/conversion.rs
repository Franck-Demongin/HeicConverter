use std::fs;
use std::path::{Path, PathBuf};

use heic_rs::PixelLayout;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb, Rgba};

use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Jpeg { quality: u8 },
    Png,
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionOptions {
    pub format: OutputFormat,
    pub apply_transforms: bool,
    pub max_pixels: u64,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Jpeg { quality: 90 },
            apply_transforms: true,
            max_pixels: 1 << 28,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionResult {
    pub input: PathBuf,
    pub output: PathBuf,
    pub width: u32,
    pub height: u32,
    pub output_bytes: u64,
    pub has_alpha: bool,
    pub sixteen_bit: bool,
}

pub fn convert_file(
    input: &Path,
    output: &Path,
    options: &ConversionOptions,
) -> Result<ConversionResult, Error> {
    if let OutputFormat::Jpeg { quality } = options.format
        && !(1..=100).contains(&quality)
    {
        return Err(Error::Heic(format!(
            "qualite JPEG invalide : {quality} (attendu 1..=100)"
        )));
    }

    let bytes = fs::read(input)?;
    let info = probe(&bytes)?;
    let total_pixels = (info.width as u64) * (info.height as u64);
    if total_pixels > options.max_pixels {
        return Err(Error::TooManyPixels {
            got: total_pixels,
            limit: options.max_pixels,
        });
    }

    let decoded = decode(&bytes, options.max_pixels, options.apply_transforms)?;
    let width = decoded.width;
    let height = decoded.height;
    let has_alpha = decoded.has_alpha;
    let sixteen_bit = decoded.sixteen_bit;
    let image = build_image(&decoded)?;

    encode(&image, output, options)?;
    let output_bytes = fs::metadata(output)
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    Ok(ConversionResult {
        input: input.to_path_buf(),
        output: output.to_path_buf(),
        width,
        height,
        output_bytes,
        has_alpha,
        sixteen_bit,
    })
}

#[derive(Debug, Clone, Copy)]
struct HeicInfo {
    width: u32,
    height: u32,
    has_alpha: bool,
    sixteen_bit: bool,
}

#[derive(Debug, Clone)]
struct DecodedImage {
    width: u32,
    height: u32,
    has_alpha: bool,
    sixteen_bit: bool,
    data: Vec<u8>,
}

fn probe(bytes: &[u8]) -> Result<HeicInfo, Error> {
    guarded(|| probe_unguarded(bytes))
}

fn probe_unguarded(bytes: &[u8]) -> Result<HeicInfo, Error> {
    let info = heic_rs::probe(bytes).map_err(err)?;
    Ok(HeicInfo {
        width: info.width,
        height: info.height,
        has_alpha: info.has_alpha,
        sixteen_bit: info.bit_depth > 8,
    })
}

fn decode(bytes: &[u8], max_pixels: u64, apply_transforms: bool) -> Result<DecodedImage, Error> {
    guarded(|| decode_unguarded(bytes, max_pixels, apply_transforms))
}

fn decode_unguarded(
    bytes: &[u8],
    max_pixels: u64,
    apply_transforms: bool,
) -> Result<DecodedImage, Error> {
    let info = probe_unguarded(bytes)?;
    let pixel_layout = match (info.has_alpha, info.sixteen_bit) {
        (true, true) => PixelLayout::Rgba16,
        (true, false) => PixelLayout::Rgba8,
        (false, true) => PixelLayout::Rgb16,
        (false, false) => PixelLayout::Rgb8,
    };
    let raw = heic_rs::DecodeOptions::default()
        .with_layout(pixel_layout)
        .with_max_pixels(Some(max_pixels))
        .with_transforms(apply_transforms)
        .with_alpha(info.has_alpha);
    let decoded = heic_rs::decode(bytes, &raw).map_err(err)?;
    Ok(DecodedImage {
        width: decoded.width,
        height: decoded.height,
        has_alpha: info.has_alpha,
        sixteen_bit: info.sixteen_bit,
        data: decoded.data,
    })
}

fn build_image(decoded: &DecodedImage) -> Result<DynamicImage, Error> {
    let width = decoded.width;
    let height = decoded.height;
    if decoded.sixteen_bit {
        let mut samples = Vec::with_capacity(decoded.data.len() / 2);
        for chunk in decoded.data.as_chunks::<2>().0 {
            samples.push(u16::from_le_bytes(*chunk));
        }
        if decoded.has_alpha {
            let buffer = ImageBuffer::<Rgba<u16>, _>::from_raw(width, height, samples)
                .ok_or_else(|| Error::Heic("tampon image invalide".into()))?;
            Ok(DynamicImage::ImageRgba16(buffer))
        } else {
            let buffer = ImageBuffer::<Rgb<u16>, _>::from_raw(width, height, samples)
                .ok_or_else(|| Error::Heic("tampon image invalide".into()))?;
            Ok(DynamicImage::ImageRgb16(buffer))
        }
    } else if decoded.has_alpha {
        let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, decoded.data.clone())
            .ok_or_else(|| Error::Heic("tampon image invalide".into()))?;
        Ok(DynamicImage::ImageRgba8(buffer))
    } else {
        let buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, decoded.data.clone())
            .ok_or_else(|| Error::Heic("tampon image invalide".into()))?;
        Ok(DynamicImage::ImageRgb8(buffer))
    }
}

fn encode(image: &DynamicImage, output: &Path, options: &ConversionOptions) -> Result<(), Error> {
    match options.format {
        OutputFormat::Jpeg { quality } => {
            let rgb = image.to_rgb8();
            let (width, height) = (rgb.width(), rgb.height());
            let mut buffer = Vec::new();
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);
            encoder
                .encode_image(&rgb)
                .map_err(|error| Error::Heic(format!("encodage JPEG : {error}")))?;
            fs::write(output, buffer)?;
            let _ = (width, height);
            Ok(())
        }
        OutputFormat::Png => {
            image
                .save_with_format(output, ImageFormat::Png)
                .map_err(|error| Error::Heic(format!("encodage PNG : {error}")))?;
            Ok(())
        }
    }
}

fn err(error: heic_rs::Error) -> Error {
    match error {
        heic_rs::Error::Unsupported(what) => Error::Heic(format!("HEIC non supporte : {what}")),
        heic_rs::Error::PixelLimit { .. } | heic_rs::Error::BoxTooLarge { .. } => {
            Error::Heic(format!("limite depassee : {error}"))
        }
        _ => Error::Heic(format!("HEIC malformed : {error}")),
    }
}

fn guarded<T>(f: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .unwrap_or_else(|_| Err(Error::Heic("le decodeur HEIC a echoue".into())))
}
