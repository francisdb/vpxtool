use image::{DynamicImage, RgbImage};
use std::collections::HashSet;
use std::path::Path;
use xcap::XCapError;

pub(crate) fn capture_vpinball_windows(captures_path: &Path) -> Result<i8, XCapError> {
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
            rgb_image
                .save(image_path)
                .map_err(|e| XCapError::new(format!("Unable to save image: {}", e)))?;
            captured += 1;
        }
    }
    Ok(captured)
}
