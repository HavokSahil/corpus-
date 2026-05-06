//! Perspective correction via 4-point homography.
//!
//! Given the four corners of a detected document and the original image,
//! this module computes a perspective warp that produces a flat, top-down
//! view of the document.
//!
//! # How it works
//!
//! A **homography** is a 3×3 matrix **H** that maps points from one plane
//! to another:
//!
//! ```text
//!     ┌ x' ┐       ┌ h00 h01 h02 ┐   ┌ x ┐
//!     │ y' │ = λ ·  │ h10 h11 h12 │ · │ y │
//!     └ 1  ┘       └ h20 h21 h22 ┘   └ 1 ┘
//! ```
//!
//! Given 4 source points (the detected corners) and 4 destination points
//! (a rectangle), OpenCV's `get_perspective_transform` solves for **H**,
//! and `warp_perspective` applies it to every pixel.

use image::DynamicImage;
use opencv::core::{Mat, Point2f, Size, Vector};
use opencv::imgproc;

use crate::config::PipelineConfig;
use crate::convert;
use crate::detect::DocumentCorners;
use crate::error::Result;

/// Warp the original image so that the document fills a rectangle.
///
/// # Arguments
///
/// * `img`     — the original (uncropped) image
/// * `corners` — four document corners in order: TL, TR, BR, BL
/// * `config`  — pipeline config (for debug output)
///
/// # Returns
///
/// A new `DynamicImage` containing the perspective-corrected document.
pub fn warp_perspective(
    img: &DynamicImage,
    corners: &DocumentCorners,
    config: &PipelineConfig,
) -> Result<DynamicImage> {
    let [tl, tr, br, bl] = *corners;

    // -----------------------------------------------------------------------
    // Compute output dimensions.
    //
    // The width is the maximum of the top edge and bottom edge lengths.
    // The height is the maximum of the left edge and right edge lengths.
    // -----------------------------------------------------------------------
    let width_top = euclidean_dist(tl, tr);
    let width_bottom = euclidean_dist(bl, br);
    let max_width = width_top.max(width_bottom).ceil() as i32;

    let height_left = euclidean_dist(tl, bl);
    let height_right = euclidean_dist(tr, br);
    let max_height = height_left.max(height_right).ceil() as i32;

    log::info!("warp output dimensions: {max_width} × {max_height}");

    // -----------------------------------------------------------------------
    // Define destination rectangle corners.
    // -----------------------------------------------------------------------
    let dst_corners: [Point2f; 4] = [
        Point2f::new(0.0, 0.0),               // TL
        Point2f::new(max_width as f32, 0.0),   // TR
        Point2f::new(max_width as f32, max_height as f32), // BR
        Point2f::new(0.0, max_height as f32),  // BL
    ];

    // -----------------------------------------------------------------------
    // Build the source and destination point vectors for OpenCV.
    // -----------------------------------------------------------------------
    let src_vec: Vector<Point2f> = Vector::from_slice(corners);
    let dst_vec: Vector<Point2f> = Vector::from_slice(&dst_corners);

    // -----------------------------------------------------------------------
    // Compute the 3×3 homography matrix H.
    //
    // H satisfies: dst = H · src  (in homogeneous coordinates)
    // -----------------------------------------------------------------------
    let h_matrix = imgproc::get_perspective_transform(&src_vec, &dst_vec, opencv::core::DECOMP_LU)?;

    // -----------------------------------------------------------------------
    // Convert the original image to an OpenCV Mat and apply the warp.
    // -----------------------------------------------------------------------
    let src_mat = convert::dynamic_image_to_mat(img)?;
    let mut dst_mat = Mat::default();

    imgproc::warp_perspective(
        &src_mat,
        &mut dst_mat,
        &h_matrix,
        Size::new(max_width, max_height),
        imgproc::INTER_LINEAR,                  // bilinear interpolation
        opencv::core::BORDER_CONSTANT,          // fill border with black
        opencv::core::Scalar::default(),
    )?;

    // -----------------------------------------------------------------------
    // Convert back to the `image` crate representation.
    // -----------------------------------------------------------------------
    let result = convert::mat_to_dynamic_image(&dst_mat)?;

    config.save_debug_image(&result, "03_warped.png")?;

    Ok(result)
}

/// Euclidean distance between two 2D points.
fn euclidean_dist(a: Point2f, b: Point2f) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}
