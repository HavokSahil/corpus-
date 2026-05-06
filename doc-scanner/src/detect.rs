//! Document boundary detection.
//!
//! Pipeline: grayscale → Gaussian blur → Canny edges → dilate → contour
//! detection (OpenCV) → find largest quadrilateral → order corners.
//!
//! This module uses `imageproc` for edge detection and `opencv` only for
//! contour detection, which is significantly more robust than a pure-Rust
//! implementation.

use image::DynamicImage;
use imageproc::edges::canny;
use imageproc::filter::gaussian_blur_f32;
use imageproc::morphology::dilate;
use imageproc::distance_transform::Norm;

use opencv::core::{Point, Point2f, Vector};
use opencv::imgproc;

use crate::config::PipelineConfig;
use crate::convert;
use crate::error::{PipelineError, Result};

/// Detected document corners, ordered: top-left, top-right, bottom-right, bottom-left.
pub type DocumentCorners = [Point2f; 4];

/// Detect the document region in the input image and return its four corners.
///
/// # Algorithm
///
/// 1. Convert to grayscale.
/// 2. Gaussian blur (σ = 2.0) to suppress noise.
/// 3. Canny edge detection with configurable thresholds.
/// 4. Morphological dilation to close small gaps in edges.
/// 5. Find external contours via OpenCV.
/// 6. Approximate each contour to a polygon; pick the largest quadrilateral.
/// 7. Order the corners consistently (TL, TR, BR, BL).
pub fn find_document(
    img: &DynamicImage,
    config: &PipelineConfig,
) -> Result<DocumentCorners> {
    // Step 0: Downscale for faster detection (especially Gaussian blur).
    let detect_width = 800.0;
    let scale = if img.width() as f32 > detect_width {
        detect_width / img.width() as f32
    } else {
        1.0
    };

    let small = if scale < 1.0 {
        img.resize(
            (img.width() as f32 * scale) as u32,
            (img.height() as f32 * scale) as u32,
            image::imageops::FilterType::Triangle,
        )
    } else {
        img.clone()
    };

    // Step 1: Grayscale
    let gray = small.to_luma8();

    // Step 2: Gaussian blur (σ = 2.0)
    let blurred = gaussian_blur_f32(&gray, 2.0);

    // Step 3: Canny edge detection
    let edges = canny(&blurred, config.canny_low, config.canny_high);

    // Save debug image: edges
    config.save_debug_image(
        &DynamicImage::ImageLuma8(edges.clone()),
        "01_edges.png",
    )?;

    // Step 4: Dilate to close small gaps in edge contours.
    let dilated = dilate(&edges, Norm::LInf, 1);

    config.save_debug_image(
        &DynamicImage::ImageLuma8(dilated.clone()),
        "02_dilated_edges.png",
    )?;

    // Step 5: Convert to OpenCV Mat for contour detection
    let edge_mat = convert::gray_image_to_mat(&dilated)?;

    let mut contours = Vector::<Vector<Point>>::new();
    let mut hierarchy = opencv::core::Mat::default();

    imgproc::find_contours_with_hierarchy(
        &edge_mat,
        &mut contours,
        &mut hierarchy,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        Point::new(0, 0),
    ).map_err(|e| PipelineError::OpenCvError(e.to_string()))?;

    log::info!("found {} contours", contours.len());

    let total_area = small.width() as f64 * small.height() as f64;

    // Step 6: Sort contours by area (descending)
    let mut contour_areas: Vec<(usize, f64)> = (0..contours.len())
        .map(|i| {
            let area = imgproc::contour_area(&contours.get(i).unwrap(), false).unwrap_or(0.0);
            (i, area)
        })
        .collect();

    contour_areas.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (idx, area) in &contour_areas {
        // If the largest contour is tiny (< 5% of image), ignore it
        if *area < total_area * 0.05 {
            break;
        }

        let contour = contours.get(*idx)?;
        
        // Extract the 4 extreme corners from this contour
        let small_corners = get_extreme_corners(&contour);
        
        log::info!("document found: area={area:.0}");
        
        // Scale corners back to original resolution
        return Ok([
            Point2f::new(small_corners[0].x / scale, small_corners[0].y / scale),
            Point2f::new(small_corners[1].x / scale, small_corners[1].y / scale),
            Point2f::new(small_corners[2].x / scale, small_corners[2].y / scale),
            Point2f::new(small_corners[3].x / scale, small_corners[3].y / scale),
        ]);
    }

    log::warn!("No clear document boundary found. Using entire image as fallback.");
    let w = img.width() as f32 - 1.0;
    let h = img.height() as f32 - 1.0;
    Ok([
        Point2f::new(0.0, 0.0),
        Point2f::new(w, 0.0),
        Point2f::new(w, h),
        Point2f::new(0.0, h),
    ])
}

/// Order four corners into a consistent arrangement from a set of points:
/// [top-left, top-right, bottom-right, bottom-left].
///
/// The heuristic:
/// - **Top-left** has the smallest sum (x + y).
/// - **Bottom-right** has the largest sum (x + y).
/// - **Top-right** has the smallest difference (y - x).
/// - **Bottom-left** has the largest difference (y - x).
fn get_extreme_corners(contour: &Vector<Point>) -> DocumentCorners {
    let mut pts = Vec::with_capacity(contour.len());
    for i in 0..contour.len() {
        let p = contour.get(i).unwrap();
        pts.push(Point2f::new(p.x as f32, p.y as f32));
    }

    if pts.is_empty() {
        return [Point2f::new(0.0, 0.0); 4];
    }

    let sums: Vec<f32> = pts.iter().map(|p| p.x + p.y).collect();
    let diffs: Vec<f32> = pts.iter().map(|p| p.y - p.x).collect();

    let tl_idx = sums
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;
    let br_idx = sums
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;
    let tr_idx = diffs
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;
    let bl_idx = diffs
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;

    [pts[tl_idx], pts[tr_idx], pts[br_idx], pts[bl_idx]]
}
