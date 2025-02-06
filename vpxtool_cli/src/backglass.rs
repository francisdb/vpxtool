use image::DynamicImage;
use std::io;

/// relative location of the hole in the image
#[derive(Debug, PartialEq)]
pub(crate) struct DMDHole {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
}

/// Finds the dmd hole in the image by using the following algorithm:
/// 1. Split the image in divisions*divisions parts
/// 2. For each part, calculate get the center of the part
/// 3. For each part, get the color of the center of the part
/// 4. For each part, trace a line from the center to the edge of the image in all four directions
/// 5. When the color changes (with a threshold), we have found the borders of that part
/// 6. Pick the largest hole we found (there might be duplicates)
/// 7. Only return the hole if it is wider than min_width% of the image
///
/// Returns the hole in the image if found
///
/// # Arguments
///
/// * `image` - The image to find the hole in
/// * `divisions` - The number of divisions to split the image in
/// * `min_width` - The minimum width of the hole as a percentage of the image width
/// * `max_deviation_u8` - The maximum deviation in color to consider a color change
///
pub(crate) fn find_hole(
    image: &DynamicImage,
    divisions: u8,
    min_width: u32,
    max_deviation_u8: u8,
) -> io::Result<Option<DMDHole>> {
    // TODO we could optimize this by skipping a part if is contained in the largest hole we found so far
    let image_width = image.width();
    let image_height = image.height();
    println!("Image size: {}x{}", image_width, image_height);
    let mut max_hole: Option<DMDHole> = None;
    for x in 0..divisions {
        for y in 0..divisions {
            let x1 = (x as f32 / divisions as f32) * image_width as f32;
            let y1 = (y as f32 / divisions as f32) * image_height as f32;
            let x2 = ((x + 1) as f32 / divisions as f32) * image_width as f32;
            let y2 = ((y + 1) as f32 / divisions as f32) * image_height as f32;
            let center_x = ((x1 + x2) / 2.0) as u32;
            let center_y = ((y1 + y2) / 2.0) as u32;

            let hole = find_hole_from(image, center_x, center_y, max_deviation_u8)?;

            let hole_width = hole.x2 - hole.x1;
            let hole_height = hole.y2 - hole.y1;
            if hole_width > min_width {
                if let Some(old_max_hole) = &max_hole {
                    if hole_width > old_max_hole.x2 - old_max_hole.x1
                        && hole_height > old_max_hole.y2 - old_max_hole.y1
                    {
                        max_hole = Some(hole);
                    }
                } else {
                    max_hole = Some(hole);
                }
            }
        }
    }

    Ok(max_hole)
}

fn find_hole_from(
    image: &DynamicImage,
    center_x: u32,
    center_y: u32,
    max_deviation_u8: u8,
) -> io::Result<DMDHole> {
    let image_width = image.width();
    let image_height = image.height();
    let rgba_image = match image.as_rgba8() {
        Some(rgba_image) => rgba_image,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Image is not in RGBA format",
            ));
        }
    };
    let center_color = rgba_image.get_pixel(center_x, center_y);
    // println!("Color at ({}, {}): {:?}", center_x, center_y, color);

    let mut left = center_x;
    while left > 0 {
        let color_x = rgba_image.get_pixel(left, center_y);
        if !color_within_deviation(center_color, color_x, max_deviation_u8) {
            left += 1;
            break;
        }
        left -= 1;
    }
    let mut right = center_x;
    while right < image_width {
        let color_x = rgba_image.get_pixel(right, center_y);
        if !color_within_deviation(center_color, color_x, max_deviation_u8) {
            right -= 1;
            break;
        }
        right += 1;
    }
    let mut top = center_y;
    while top > 0 {
        let color_y = rgba_image.get_pixel(center_x, top);
        if !color_within_deviation(center_color, color_y, max_deviation_u8) {
            top += 1;
            break;
        }
        top -= 1;
    }
    let mut bottom = center_y;
    while bottom < image_height {
        let color_y = rgba_image.get_pixel(center_x, bottom);
        if !color_within_deviation(center_color, color_y, max_deviation_u8) {
            bottom -= 1;
            break;
        }
        bottom += 1;
    }
    let hole = DMDHole {
        x1: left,
        y1: top,
        x2: right,
        y2: bottom,
    };
    Ok(hole)
}

fn color_within_deviation(c1: &image::Rgba<u8>, c2: &image::Rgba<u8>, max_deviation: u8) -> bool {
    let diff = |a: u8, b: u8| a.abs_diff(b) as u32;
    let total_deviation: u32 =
        c1.0.iter()
            .zip(c2.0.iter())
            .map(|(a, b)| diff(*a, *b))
            .sum();
    total_deviation <= max_deviation as u32 * 4
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;
    use pretty_assertions::assert_eq;
    use rand::Rng;

    #[test]
    fn test_find_hole() {
        let width = 320;
        let height = 200;
        let mut image = noise_image(width, height);
        clear_square(
            &mut image,
            100,
            50,
            100,
            50,
            image::Rgba([0xFF, 0xAA, 0x22, 255]),
        );
        let dynamic_image = DynamicImage::ImageRgba8(image);

        let hole = find_hole(&dynamic_image, 10, 50, 1).unwrap();
        let expected = Some(DMDHole {
            x1: 100,
            y1: 50,
            x2: 199,
            y2: 99,
        });
        assert_eq!(hole, expected);
    }

    #[test]
    fn test_find_hole_no_hole() {
        let width = 320;
        let height = 200;
        let image = noise_image(width, height);
        let dynamic_image = DynamicImage::ImageRgba8(image);

        let hole = find_hole(&dynamic_image, 10, 10, 1).unwrap();
        assert_eq!(hole, None);
    }

    #[test]
    fn test_find_whole_image_with_deviation_max() {
        let width = 320;
        let height = 200;
        let image = noise_image(width, height);
        let dynamic_image = DynamicImage::ImageRgba8(image);

        let hole = find_hole(&dynamic_image, 10, 100, 255).unwrap();
        let expected = Some(DMDHole {
            x1: 0,
            y1: 0,
            x2: width,
            y2: height,
        });
        assert_eq!(hole, expected);
    }

    fn noise_image(width: u32, height: u32) -> RgbaImage {
        let dynamic_image = DynamicImage::new_rgba8(width, height);
        let mut image = dynamic_image.to_rgba8();
        let mut rng = rand::rng();
        for x in 0..width {
            for y in 0..height {
                let random_color = image::Rgba([rng.random(), rng.random(), rng.random(), 255]);
                image.put_pixel(x, y, random_color);
            }
        }
        image
    }

    fn clear_square(
        image: &mut RgbaImage,
        x1: u32,
        y1: u32,
        width: u32,
        height: u32,
        color: image::Rgba<u8>,
    ) {
        for x in x1..x1 + width {
            for y in y1..y1 + height {
                image.put_pixel(x, y, color);
            }
        }
    }
}
