use anyhow::Result;
use image::{DynamicImage, GrayImage, Luma};
use imageproc::contrast::{threshold, ThresholdType};

/// Preprocess image to improve OCR accuracy
pub fn preprocess_image(image: DynamicImage) -> Result<DynamicImage> {
    // Convert to grayscale
    let gray = image.to_luma8();

    // Apply adaptive thresholding for better text recognition
    let processed = apply_adaptive_threshold(&gray);

    // Apply noise reduction
    let denoised = denoise(&processed);

    Ok(DynamicImage::ImageLuma8(denoised))
}

fn apply_adaptive_threshold(image: &GrayImage) -> GrayImage {
    // Calculate Otsu's threshold
    let threshold_value = otsu_threshold(image);
    threshold(image, threshold_value, ThresholdType::Binary)
}

fn otsu_threshold(image: &GrayImage) -> u8 {
    // Calculate histogram
    let mut histogram = [0u32; 256];
    for pixel in image.pixels() {
        histogram[pixel[0] as usize] += 1;
    }

    let total_pixels = (image.width() * image.height()) as f64;
    let mut sum = 0.0;
    for (i, &count) in histogram.iter().enumerate() {
        sum += i as f64 * count as f64;
    }

    let mut sum_background = 0.0;
    let mut weight_background = 0.0;
    let mut max_variance = 0.0;
    let mut threshold = 0u8;

    for (t, &count) in histogram.iter().enumerate() {
        weight_background += count as f64;
        if weight_background == 0.0 {
            continue;
        }

        let weight_foreground = total_pixels - weight_background;
        if weight_foreground == 0.0 {
            break;
        }

        sum_background += t as f64 * count as f64;

        let mean_background = sum_background / weight_background;
        let mean_foreground = (sum - sum_background) / weight_foreground;

        let variance = weight_background * weight_foreground * 
                      (mean_background - mean_foreground).powi(2);

        if variance > max_variance {
            max_variance = variance;
            threshold = t as u8;
        }
    }

    threshold
}

fn denoise(image: &GrayImage) -> GrayImage {
    // Apply median filter for noise reduction
    let (width, height) = image.dimensions();
    let mut output = GrayImage::new(width, height);

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let mut neighbors = Vec::with_capacity(9);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let px = (x as i32 + dx) as u32;
                    let py = (y as i32 + dy) as u32;
                    neighbors.push(image.get_pixel(px, py)[0]);
                }
            }
            neighbors.sort_unstable();
            output.put_pixel(x, y, Luma([neighbors[4]]));
        }
    }

    // Copy edges from original
    for x in 0..width {
        output.put_pixel(x, 0, *image.get_pixel(x, 0));
        output.put_pixel(x, height - 1, *image.get_pixel(x, height - 1));
    }
    for y in 0..height {
        output.put_pixel(0, y, *image.get_pixel(0, y));
        output.put_pixel(width - 1, y, *image.get_pixel(width - 1, y));
    }

    output
}
