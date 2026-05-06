//! Conversion utilities between the `image` crate types and OpenCV `Mat`.
//!
//! The pipeline uses `image`/`imageproc` as the primary representation and
//! only converts to OpenCV `Mat` for contour detection and perspective warp.
//! These functions live at the boundary between the two worlds.

use image::{DynamicImage, GrayImage, RgbImage};
use opencv::core::{Mat, MatTraitConst, CV_8UC1, CV_8UC3};
use opencv::prelude::*;

use crate::error::{PipelineError, Result};

// ---------------------------------------------------------------------------
// image → OpenCV
// ---------------------------------------------------------------------------

/// Convert an `image::GrayImage` to an OpenCV `Mat` (CV_8UC1).
///
/// The pixel data is copied into the Mat to ensure OpenCV owns the memory.
pub fn gray_image_to_mat(img: &GrayImage) -> Result<Mat> {
    let (w, h) = img.dimensions();
    let data = img.as_raw();

    // Safety: we create the Mat from a contiguous slice with known dimensions.
    // `Mat::from_slice_rows_cols` copies the data.
    let mat = unsafe {
        Mat::new_rows_cols_with_data_unsafe_def(
            h as i32,
            w as i32,
            CV_8UC1,
            data.as_ptr() as *mut std::ffi::c_void,
        )?
    };

    // Clone to own the data (the above Mat borrows the slice).
    Ok(mat.clone())
}

/// Convert an `image::RgbImage` to an OpenCV `Mat` (CV_8UC3, BGR layout).
///
/// OpenCV expects BGR channel order, so we swap R↔B during the copy.
pub fn rgb_image_to_mat(img: &RgbImage) -> Result<Mat> {
    let (w, h) = img.dimensions();

    // Allocate a BGR Mat and fill it row-by-row.
    let mut mat = Mat::zeros(h as i32, w as i32, CV_8UC3)?.to_mat()?;

    for y in 0..h {
        for x in 0..w {
            let px = img.get_pixel(x, y);
            let bgr: [u8; 3] = [px[2], px[1], px[0]]; // RGB → BGR
            // Set pixel in the Mat.
            *mat.at_2d_mut::<opencv::core::Vec3b>(y as i32, x as i32)? =
                opencv::core::Vec3b::from(bgr);
        }
    }

    Ok(mat)
}

/// Convert a `DynamicImage` to an OpenCV BGR `Mat`.
pub fn dynamic_image_to_mat(img: &DynamicImage) -> Result<Mat> {
    rgb_image_to_mat(&img.to_rgb8())
}

// ---------------------------------------------------------------------------
// OpenCV → image
// ---------------------------------------------------------------------------

/// Convert an OpenCV `Mat` (CV_8UC3, BGR) back to an `image::RgbImage`.
pub fn mat_to_rgb_image(mat: &Mat) -> Result<RgbImage> {
    let rows = mat.rows();
    let cols = mat.cols();

    if rows <= 0 || cols <= 0 {
        return Err(PipelineError::OpenCvError(
            "Mat has zero or negative dimensions".into(),
        ));
    }

    let mut img = RgbImage::new(cols as u32, rows as u32);

    for y in 0..rows {
        for x in 0..cols {
            let bgr = mat.at_2d::<opencv::core::Vec3b>(y, x)?;
            // BGR → RGB
            *img.get_pixel_mut(x as u32, y as u32) =
                image::Rgb([bgr[2], bgr[1], bgr[0]]);
        }
    }

    Ok(img)
}

/// Convert an OpenCV `Mat` (CV_8UC1) to an `image::GrayImage`.
pub fn mat_to_gray_image(mat: &Mat) -> Result<GrayImage> {
    let rows = mat.rows();
    let cols = mat.cols();

    if rows <= 0 || cols <= 0 {
        return Err(PipelineError::OpenCvError(
            "Mat has zero or negative dimensions".into(),
        ));
    }

    let mut img = GrayImage::new(cols as u32, rows as u32);

    for y in 0..rows {
        for x in 0..cols {
            let val = *mat.at_2d::<u8>(y, x)?;
            *img.get_pixel_mut(x as u32, y as u32) = image::Luma([val]);
        }
    }

    Ok(img)
}

/// Convert an OpenCV `Mat` to a `DynamicImage`, auto-detecting channel count.
pub fn mat_to_dynamic_image(mat: &Mat) -> Result<DynamicImage> {
    let channels = mat.channels();
    match channels {
        1 => Ok(DynamicImage::ImageLuma8(mat_to_gray_image(mat)?)),
        3 => Ok(DynamicImage::ImageRgb8(mat_to_rgb_image(mat)?)),
        n => Err(PipelineError::OpenCvError(format!(
            "unsupported channel count: {n}"
        ))),
    }
}
