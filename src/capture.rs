use image::{DynamicImage, RgbImage};
use std::collections::HashSet;
use std::path::Path;

use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum CaptureError {
    // #[error("{0}")]
    // Error(String),
    #[error(transparent)]
    StdIOError(#[from] std::io::Error),
    #[error(transparent)]
    XCapError(#[from] xcap::XCapError),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
}

pub(crate) fn capture_vpinball_windows(captures_path: &Path) -> Result<i8, CaptureError> {
    // TODO we probably want a map that stores to what file the capture belongs
    let titles: HashSet<&str> = vec![
        "Visual Pinball Player",
        "Visual Pinball - Score",
        "PUPPlayfield",
        "PUPDMD",
        "PUPBackglass",
        "PUPFullDMD",
        "PUPTopper",
        "B2SBackglass",
        "B2SDMD",
        "PinMAME",
        "FlexDMD",
    ]
    .into_iter()
    .collect();

    let windows = xcap::Window::all()?;
    let mut captured = 0;
    for window in windows {
        if window.is_minimized()? {
            continue;
        }
        let title = window.title()?;
        if titles.contains(title.as_str()) {
            println!(
                "Window: {} at {},{} {}x{} minimized: {} maximized: {}",
                window.title()?,
                window.x()?,
                window.y()?,
                window.width()?,
                window.height()?,
                window.is_minimized()?,
                window.is_maximized()?
            );
            let image = window.capture_image()?;

            let image_path = captures_path.join(title).with_extension("png");
            println!("Saving to {:?}", image_path);
            // drop the alpha channel to reduce file size
            let rgb_image: RgbImage = DynamicImage::from(image).to_rgb8();
            rgb_image.save(image_path)?;
            captured += 1;
        }
    }
    Ok(captured)
}
