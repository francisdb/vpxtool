// src/ui/gradient.rs
use anyhow::Result;
use sdl3::pixels::Color;
use sdl3::rect::Rect;

pub fn draw_gradient_box(
    canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
    rect: Rect,
    start_color: Color,
    end_color: Color,
) -> Result<()> {
    let steps = rect.height();

    canvas.set_blend_mode(sdl3::render::BlendMode::Blend);

    for i in 0..steps {
        let normalized = i as f32 / (steps as f32 - 1.0);
        let progress = normalized * normalized * (3.0 - 2.0 * normalized);

        let r = start_color.r as f32 + progress * (end_color.r as f32 - start_color.r as f32);
        let g = start_color.g as f32 + progress * (end_color.g as f32 - start_color.g as f32);
        let b = start_color.b as f32 + progress * (end_color.b as f32 - start_color.b as f32);
        let a = start_color.a as f32 + progress * (end_color.a as f32 - start_color.a as f32);

        canvas.set_draw_color(Color::RGBA(r as u8, g as u8, b as u8, a as u8));
        canvas.fill_rect(Rect::new(rect.x(), rect.y() + i as i32, rect.width(), 1))?;
    }

    Ok(())
}
