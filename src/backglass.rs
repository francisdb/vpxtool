use image::{DynamicImage, RgbaImage};
use std::io;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) struct Vec2 {
    pub(crate) x: u32,
    pub(crate) y: u32,
}

impl Vec2 {
    pub fn new(x: u32, y: u32) -> Vec2 {
        Vec2 { x, y }
    }
}

/// relative location of the hole in the image
#[derive(Debug, PartialEq)]
pub(crate) struct DMDHole {
    pub(crate) pos: Vec2,
    pub(crate) dim: Vec2,
    pub(crate) parent_dim: Vec2,
}

impl DMDHole {
    pub(crate) fn new(
        x1: u32,
        y1: u32,
        x2: u32,
        y2: u32,
        parent_width: u32,
        parent_height: u32,
    ) -> DMDHole {
        DMDHole {
            pos: Vec2 { x: x1, y: y1 },
            dim: Vec2 {
                x: x2 - x1 + 1,
                y: y2 - y1 + 1,
            },
            parent_dim: Vec2 {
                x: parent_width,
                y: parent_height,
            },
        }
    }

    pub fn width(&self) -> u32 {
        self.dim.x
    }

    pub fn height(&self) -> u32 {
        self.dim.y
    }

    pub fn x(&self) -> u32 {
        self.pos.x
    }

    pub fn y(&self) -> u32 {
        self.pos.y
    }

    #[allow(dead_code)]
    pub fn parent_width(&self) -> u32 {
        self.parent_dim.x
    }

    #[allow(dead_code)]
    pub fn parent_height(&self) -> u32 {
        self.parent_dim.y
    }

    pub fn scale_to_parent(&self, width: u32, height: u32) -> DMDHole {
        let x = (self.pos.x as f32 / self.parent_dim.x as f32 * width as f32) as u32;
        let y = (self.pos.y as f32 / self.parent_dim.y as f32 * height as f32) as u32;
        let dim_x = (self.dim.x as f32 / self.parent_dim.x as f32 * width as f32) as u32;
        let dim_y = (self.dim.y as f32 / self.parent_dim.y as f32 * height as f32) as u32;
        DMDHole {
            pos: Vec2 { x, y },
            dim: Vec2 { x: dim_x, y: dim_y },
            parent_dim: Vec2 {
                x: width,
                y: height,
            },
        }
    }
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

            if hole.width() > min_width {
                if let Some(old_max_hole) = &max_hole {
                    if hole.width() * hole.height() > old_max_hole.width() * old_max_hole.height() {
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
    let center: Vec2 = Vec2 {
        x: center_x,
        y: center_y,
    };
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

    let mut left = center_x;
    while left > 0 {
        left -= 1;
        let color_x = rgba_image.get_pixel(left, center_y);
        if !color_within_deviation(center_color, color_x, max_deviation_u8) {
            left += 1;
            break;
        }
    }
    let mut right = center_x;
    while right < image_width - 1 {
        right += 1;
        let color_x = rgba_image.get_pixel(right, center_y);
        if !color_within_deviation(center_color, color_x, max_deviation_u8) {
            right -= 1;
            break;
        }
    }
    let mut top = center_y;
    while top > 0 {
        top -= 1;
        let color_y = rgba_image.get_pixel(center_x, top);
        if !color_within_deviation(center_color, color_y, max_deviation_u8) {
            top += 1;
            break;
        }
    }
    let mut bottom = center_y;
    while bottom < image_height - 1 {
        bottom += 1;
        let color_y = rgba_image.get_pixel(center_x, bottom);
        if !color_within_deviation(center_color, color_y, max_deviation_u8) {
            bottom -= 1;
            break;
        }
    }

    // Now we do an outward from the center toward the corners check to account for
    // shamfered/filleted corners and shrink the hole accordingly.

    let top_left = trace_line(
        rgba_image,
        center,
        Vec2::new(left, top),
        center_color,
        max_deviation_u8,
    );
    let top_right = trace_line(
        rgba_image,
        center,
        Vec2::new(right, top),
        center_color,
        max_deviation_u8,
    );
    let bottom_left = trace_line(
        rgba_image,
        center,
        Vec2::new(left, bottom),
        center_color,
        max_deviation_u8,
    );

    let bottom_right = trace_line(
        rgba_image,
        center,
        Vec2::new(right, bottom),
        center_color,
        max_deviation_u8,
    );

    let left = top_left.x.max(bottom_left.x);
    let right = top_right.x.min(bottom_right.x);
    let top = top_left.y.max(top_right.y);
    let bottom = bottom_left.y.min(bottom_right.y);

    let hole = DMDHole::new(left, top, right, bottom, image_width, image_height);
    Ok(hole)
}

fn trace_line(
    rgba_image: &RgbaImage,
    start: Vec2,
    end: Vec2,
    color: &image::Rgba<u8>,
    max_deviation_u8: u8,
) -> Vec2 {
    let mut current = end;
    for point in LinePixelIterator::new(start, end) {
        let current_color = rgba_image.get_pixel(point.x, point.y);
        if !color_within_deviation(current_color, color, max_deviation_u8) {
            break;
        }
        current = point;
    }
    current
}

struct LinePixelIterator {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    dx: i32,
    dy: i32,
    sx: i32,
    sy: i32,
    err: i32,
    done: bool,
}

impl LinePixelIterator {
    fn new(from: Vec2, to: Vec2) -> Self {
        let x0 = from.x as i32;
        let y0 = from.y as i32;
        let x1 = to.x as i32;
        let y1 = to.y as i32;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let err = dx - dy;
        LinePixelIterator {
            x0,
            y0,
            x1,
            y1,
            dx,
            dy,
            sx,
            sy,
            err,
            done: false,
        }
    }
}

impl Iterator for LinePixelIterator {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let point = Vec2 {
            x: self.x0 as u32,
            y: self.y0 as u32,
        };

        if self.x0 == self.x1 && self.y0 == self.y1 {
            self.done = true;
        } else {
            let e2 = 2 * self.err;
            if e2 > -self.dy {
                self.err -= self.dy;
                self.x0 += self.sx;
            }
            if e2 < self.dx {
                self.err += self.dx;
                self.y0 += self.sy;
            }
        }

        Some(point)
    }
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
    fn test_find_hole_from() {
        let image_width = 20;
        let image_height = 16;
        let mut image = noise_image(image_width, image_height);
        clear_square(
            &mut image,
            5,
            4,
            10,
            8,
            image::Rgba([0xFF, 0xAA, 0x22, 255]),
        );
        let dynamic_image = DynamicImage::ImageRgba8(image);

        let hole = find_hole_from(&dynamic_image, image_width / 2, image_height / 2, 0).unwrap();
        let expected = DMDHole::new(5, 4, 14, 11, image_width, image_height);
        assert_eq!(hole, expected);
    }

    #[test]
    fn test_find_hole_from_with_inward_corners() {
        // we create an image with a cross like hole to force the algorithm to find the inward corners
        let image_width = 100;
        let image_height = 100;
        let mut image = noise_image(image_width, image_height);
        let black = image::Rgba([0x00, 0x00, 0x00, 255]);
        clear_square(&mut image, 10, 20, 80, 60, black);
        clear_square(&mut image, 20, 10, 60, 80, black);
        let dynamic_image = DynamicImage::ImageRgba8(image);

        // write image to disk
        //dynamic_image.save("test_find_hole_from.png").unwrap();

        let hole = find_hole_from(&dynamic_image, image_width / 2, image_height / 2, 0).unwrap();
        assert_eq!(hole.width(), 60);
        assert_eq!(hole.height(), 60);
        assert_eq!(hole.x(), 20);
        assert_eq!(hole.y(), 20);
    }

    #[test]
    fn test_find_hole() {
        let image_width = 320;
        let image_height = 200;
        let mut image = noise_image(image_width, image_height);
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
        let expected = Some(DMDHole::new(100, 50, 199, 99, image_width, image_height));
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
        let image_width = 320;
        let image_height = 200;
        let image = noise_image(image_width, image_height);
        let dynamic_image = DynamicImage::ImageRgba8(image);

        let hole = find_hole(&dynamic_image, 10, 100, 255).unwrap();
        let expected = Some(DMDHole::new(
            0,
            0,
            image_width - 1,
            image_height - 1,
            image_width,
            image_height,
        ));
        assert_eq!(hole, expected);
    }

    #[test]
    fn test_dmd_hole_1_x_1() {
        let hole = DMDHole::new(0, 0, 0, 0, 1, 1);
        assert_eq!(hole.width(), 1);
        assert_eq!(hole.height(), 1);
        assert_eq!(hole.x(), 0);
        assert_eq!(hole.y(), 0);
    }

    #[test]
    fn test_dmd_hole_scale_1_x_1_to_parent() {
        let hole = DMDHole::new(0, 0, 1, 1, 2, 2);
        let scaled_hole = hole.scale_to_parent(4, 4);
        assert_eq!(scaled_hole.width(), 4);
        assert_eq!(scaled_hole.height(), 4);
        assert_eq!(scaled_hole.x(), 0);
        assert_eq!(scaled_hole.y(), 0);
    }

    #[test]
    fn test_dmd_hole_scale_to_parent() {
        let hole = DMDHole::new(8, 8, 21, 21, 30, 30);
        let scaled_hole = hole.scale_to_parent(20, 20);
        assert_eq!(scaled_hole.width(), 9);
        assert_eq!(scaled_hole.height(), 9);
        assert_eq!(scaled_hole.x(), 5);
        assert_eq!(scaled_hole.y(), 5);
        assert_eq!(scaled_hole.parent_width(), 20);
        assert_eq!(scaled_hole.parent_height(), 20);
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
