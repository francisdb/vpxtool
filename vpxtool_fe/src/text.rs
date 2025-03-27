// src/ui/text.rs
use crate::gradient::draw_gradient_box;
use anyhow::Result;
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::render::{Texture, TextureCreator};
use sdl3::ttf::Font;
use sdl3::video::WindowContext;

const TITLE_BOTTOM_PADDING: u32 = 40;

pub fn render_text<'a>(
    font: &Font,
    text: &str,
    color: Color,
    texture_creator: &'a TextureCreator<WindowContext>,
) -> Result<Texture<'a>> {
    let surface = font.render(text).blended(color)?;
    let texture = texture_creator.create_texture_from_surface(&surface)?;
    Ok(texture)
}

pub fn draw_text_box(
    canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
    textures: &[Texture],
    text_rects: &[Rect],
) -> Result<()> {
    let (width, _height) = canvas.output_size()?;

    // Find the top-most and bottom-most points of all text rectangles
    let min_y = text_rects.iter().map(|r| r.y()).min().unwrap_or(0);
    let max_y = text_rects
        .iter()
        .map(|r| r.y() + r.height() as i32)
        .max()
        .unwrap_or(0);

    // Add some padding around the text
    let all_text_rect = Rect::new(
        0,
        min_y,
        width,
        (max_y - min_y + TITLE_BOTTOM_PADDING as i32) as u32,
    );

    let gradient_padding_top = 200;
    let gradient_rect = Rect::new(
        0,
        all_text_rect.y() - gradient_padding_top,
        width,
        all_text_rect.height() + gradient_padding_top as u32,
    );

    // Draw gradient
    canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
    draw_gradient_box(
        canvas,
        gradient_rect,
        Color::RGBA(0, 0, 0, 0),
        Color::RGBA(0, 0, 0, 230),
    )?;

    // Draw text
    for (texture, text_rect) in textures.iter().zip(text_rects.iter()) {
        canvas.copy(texture, None, *text_rect)?;
    }

    Ok(())
}
