use crate::image_processor::common::QualityMode;
use crate::image_processor::common::{
    has_transparency, is_likely_screenshot, is_webp_animation, might_contain_text,
    preprocess_image, CompressionError,
};
use image;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use webp::Encoder;
use webp_animation::prelude::*;

pub fn compress_webp_lossless(
    input_path: &Path,
    output_path: &Path,
) -> Result<(), CompressionError> {
    let img =
        image::open(input_path).map_err(|e| CompressionError::ImageProcessing(e.to_string()))?;
    let rgba_image = img.to_rgba8();
    let width = img.width();
    let height = img.height();

    let encoder = Encoder::from_rgba(&rgba_image, width, height);
    let webp_data = encoder.encode_lossless();

    std::fs::write(output_path, webp_data.to_vec())
        .map_err(|e| CompressionError::ImageProcessing(e.to_string()))?;

    Ok(())
}

pub fn compress_webp_adaptive(
    input_path: &Path,
    output_path: &Path,
    quality_level: u8,
) -> Result<(), CompressionError> {
    let img =
        image::open(input_path).map_err(|e| CompressionError::ImageProcessing(e.to_string()))?;

    let _has_transparency = has_transparency(&img);
    let is_screenshot = is_likely_screenshot(&img);
    let has_text = might_contain_text(&img);

    if is_screenshot || has_text {
        compress_webp_lossless(input_path, output_path)
    } else {
        compress_webp_with_preprocessing(input_path, output_path, quality_level)
    }
}

pub fn compress_webp_with_preprocessing(
    input_path: &Path,
    output_path: &Path,
    quality_level: u8,
) -> Result<(), CompressionError> {
    let img =
        image::open(input_path).map_err(|e| CompressionError::ImageProcessing(e.to_string()))?;

    let quality = match quality_level {
        6 => 10,
        5 => 30,
        4 => 60,
        3 => 75,
        2 => 85,
        1 => 100,
        _ => 75,
    };
    let processed_img = preprocess_image(&img, quality);

    let rgba_image = processed_img.to_rgba8();
    let width = processed_img.width();
    let height = processed_img.height();

    let encoder = Encoder::from_rgba(&rgba_image, width, height);
    let webp_data = encoder.encode(quality as f32);

    std::fs::write(output_path, webp_data.to_vec())
        .map_err(|e| CompressionError::ImageProcessing(e.to_string()))?;

    Ok(())
}

pub fn compress_webp_animation(
    input_path: &Path,
    output_path: &Path,
    quality: u8,
) -> Result<(), CompressionError> {
    let buffer = fs::read(input_path).map_err(|e| CompressionError::Io(e))?;

    let decoder = match Decoder::new(&buffer) {
        Ok(decoder) => decoder,
        Err(e) => {
            log::warn!(
                "输入文件不是WebP动画或解码失败: {}，尝试使用自适应WebP压缩",
                e
            );
            let optimize_level = ((100 - quality) / 10).min(10);
            return compress_webp_adaptive(input_path, output_path, optimize_level);
        }
    };

    let frames: Vec<_> = decoder.into_iter().collect();
    if frames.len() <= 1 {
        let optimize_level = ((100 - quality) / 10).min(10);
        return compress_webp_adaptive(input_path, output_path, optimize_level);
    }

    let dimensions = if let Some(first_frame) = frames.first() {
        first_frame.dimensions()
    } else {
        let optimize_level = ((100 - quality) / 10).min(10);
        return compress_webp_adaptive(input_path, output_path, optimize_level);
    };

    let encoding_config = if quality < 50 {
        EncodingConfig {
            quality: quality as f32,
            encoding_type: EncodingType::Lossy(LossyEncodingConfig {
                segments: 4,
                sns_strength: 50,
                filter_strength: 60,
                filter_sharpness: 0,
                filter_type: 1,
                pass: 2,
                preprocessing: true,
                partitions: 3,
                partition_limit: 100,
                alpha_compression: true,
                alpha_filtering: 2,
                alpha_quality: quality as usize,
                autofilter: true,
                use_sharp_yuv: true,
                target_size: 0,
                target_psnr: 0.0,
                show_compressed: false,
            }),
            method: 6,
        }
    } else if quality < 80 {
        EncodingConfig {
            quality: quality as f32,
            encoding_type: EncodingType::Lossy(LossyEncodingConfig {
                segments: 3,
                sns_strength: 35,
                filter_strength: 40,
                filter_sharpness: 3,
                filter_type: 1,
                pass: 1,
                preprocessing: true,
                partitions: 2,
                partition_limit: 80,
                alpha_compression: true,
                alpha_filtering: 1,
                alpha_quality: quality as usize,
                autofilter: true,
                use_sharp_yuv: true,
                target_size: 0,
                target_psnr: 0.0,
                show_compressed: false,
            }),
            method: 5,
        }
    } else {
        if quality >= 95 {
            EncodingConfig {
                quality: 100.0,
                encoding_type: EncodingType::Lossless,
                method: 6,
            }
        } else {
            EncodingConfig {
                quality: quality as f32,
                encoding_type: EncodingType::Lossy(LossyEncodingConfig {
                    segments: 2,
                    sns_strength: 25,
                    filter_strength: 20,
                    filter_sharpness: 6,
                    filter_type: 0,
                    pass: 1,
                    preprocessing: false,
                    partitions: 1,
                    partition_limit: 50,
                    alpha_compression: true,
                    alpha_filtering: 0,
                    alpha_quality: 100,
                    autofilter: true,
                    use_sharp_yuv: true,
                    target_size: 0,
                    target_psnr: 0.0,
                    show_compressed: false,
                }),
                method: 4,
            }
        }
    };

    let encoder_options = webp_animation::EncoderOptions {
        anim_params: webp_animation::AnimParams { loop_count: 0 },
        kmin: 3,
        kmax: if quality < 50 { 10 } else { 5 },
        ..Default::default()
    };

    let mut encoder = webp_animation::Encoder::new_with_options(dimensions, encoder_options)
        .map_err(|e| CompressionError::ImageProcessing(e.to_string()))?;

    encoder
        .set_default_encoding_config(encoding_config.clone())
        .map_err(|e| CompressionError::ImageProcessing(e.to_string()))?;

    // 使用rayon并行预处理所有帧数据，保存为元组(索引, 帧数据, 时间戳)
    // 这样可以在并行处理后保持原始顺序
    let processed_frames: Vec<(usize, Vec<u8>, i32)> = frames
        .par_iter()
        .enumerate()
        .map(|(i, frame)| {
            log::debug!("并行处理第{}帧", i + 1);

            // 获取帧数据和时间戳
            let mut frame_data = frame.data().to_vec(); // 转换为Vec<u8>
            let timestamp = frame.timestamp() as i32; // 转换为i32类型

            // 根据质量参数对帧数据进行预处理
            // 这些操作可以在并行中进行，提高处理速度
            if quality < 70 {
                // 低质量设置下可以进行一些预处理

                // 1. 创建图像实例以便处理
                if let Ok(img) = image::load_from_memory(&frame_data) {
                    // 清理日志，避免刷屏
                    if i == 0 || i % 10 == 0 {
                        log::debug!(
                            "预处理第{}帧，原始尺寸: {}x{}",
                            i + 1,
                            img.width(),
                            img.height()
                        );
                    }

                    // 2. 对非常低质量的帧进行尺寸缩小
                    if quality < 30 && (img.width() > 800 || img.height() > 800) {
                        // 计算新尺寸，保持宽高比
                        let scale_factor = 800.0 / img.width().max(img.height()) as f32;
                        let new_width = (img.width() as f32 * scale_factor) as u32;
                        let new_height = (img.height() as f32 * scale_factor) as u32;

                        // 调整尺寸
                        let resized = img.resize(
                            new_width,
                            new_height,
                            image::imageops::FilterType::Lanczos3,
                        );

                        // 转换回WebP格式
                        let rgba = resized.to_rgba8();
                        let encoder =
                            webp::Encoder::from_rgba(&rgba, resized.width(), resized.height());
                        let webp_data = encoder.encode(quality as f32);
                        frame_data = webp_data.to_vec();

                        if i == 0 || i % 10 == 0 {
                            log::debug!("第{}帧已调整尺寸: {}x{}", i + 1, new_width, new_height);
                        }
                    }
                    // 3. 对极低质量的帧应用轻微模糊
                    else if quality < 20 {
                        // 应用轻微模糊
                        let blurred = img.blur(0.8);

                        // 转换回WebP格式
                        let rgba = blurred.to_rgba8();
                        let encoder =
                            webp::Encoder::from_rgba(&rgba, blurred.width(), blurred.height());
                        let webp_data = encoder.encode(quality as f32);
                        frame_data = webp_data.to_vec();

                        if i == 0 || i % 10 == 0 {
                            log::debug!("第{}帧已应用模糊", i + 1);
                        }
                    }
                }
            }

            (i, frame_data, timestamp)
        })
        .collect();

    // 按原始顺序将处理后的帧添加到编码器
    for (i, frame_data, timestamp) in processed_frames.into_iter() {
        // 清理日志，避免刷屏
        if i == 0 || i % 10 == 0 || i == frames.len() - 1 {
            log::debug!("添加第{}帧到编码器 ({}/{})", i + 1, i + 1, frames.len());
        }

        // 添加帧到编码器
        encoder
            .add_frame(&frame_data, timestamp)
            .map_err(|e| CompressionError::ImageProcessing(e.to_string()))?;
    }

    // 获取最后一帧的时间戳作为最终时间戳
    // 重新创建解码器以获取最后一帧的时间戳
    let last_frame_timestamp = match Decoder::new(&buffer) {
        Ok(decoder) => {
            decoder
                .into_iter()
                .last()
                .map(|frame| frame.timestamp() + 100) // 默认显示最后一帧100ms
                .unwrap_or(1000) // 如果无法获取，默认使用1000ms
        }
        Err(_) => 1000, // 出错时使用默认值
    };

    // 完成编码并获取最终WebP数据
    let webp_data = encoder
        .finalize(last_frame_timestamp)
        .map_err(|e| CompressionError::ImageProcessing(e.to_string()))?;

    std::fs::write(output_path, webp_data).map_err(|e| CompressionError::Io(e))?;

    log::info!("WebP动画压缩完成: {}", output_path.display());
    Ok(())
}

pub fn compress_webp(
    input_path: &Path,
    output_path: &Path,
    quality_level: u8,
    mode: Option<QualityMode>,
) -> Result<(), CompressionError> {
    if is_webp_animation(input_path).map_err(CompressionError::Io)? {
        return Err(CompressionError::UnsupportedFormat(
            "Animated WebP".to_string(),
        ));
    }
    match mode {
        Some(QualityMode::Lossless) => compress_webp_lossless(input_path, output_path),
        Some(QualityMode::Lossy) => {
            compress_webp_with_preprocessing(input_path, output_path, quality_level)
        }
        None => compress_webp_with_preprocessing(input_path, output_path, quality_level),
    }
}
