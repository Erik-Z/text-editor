use sdl2::{
    self,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureQuery},
    ttf::Font,
    video::Window,
};

use crate::{settings, gap_buffer::GapBuffer};

pub fn render_text(canvas: &mut Canvas<Window>, font: &Font, text: &str, scroll_x: i32, scroll_y: i32) {
    // let lines: Vec<&str> = text.split('\n').collect();
    let lines;
    if !settings::debug_mode {
        let mut chars = text.chars();
        chars.next_back();
        lines = chars.as_str().lines();
    } else {
        lines = text.lines();
    }
    let mut y_offset = 0;

    for line in lines {
        let text_surface;
        if line.len() == 0 {
            text_surface = font
                .render(" ")
                .blended(Color::WHITE)
                .expect("Failed to render font.");
        } else {
            text_surface = font
                .render(line)
                .blended(Color::WHITE)
                .expect("Failed to render font.");
        }

        let texture_creator = canvas.texture_creator();
        let text_texture = texture_creator
            .create_texture_from_surface(&text_surface)
            .unwrap();

        let TextureQuery { width, height, .. } = text_texture.query();

        let x = 0 - scroll_x;
        let y = 0 - scroll_y + y_offset;

        let dst = Rect::new(x as i32, y as i32, width, height);
        canvas.copy(&text_texture, None, Some(dst)).unwrap();

        y_offset += height as i32;
    }
}

pub fn render_cursor(
    canvas: &mut Canvas<Window>,
    font: &Font,
    cursor_x: i32,
    cursor_y: i32,
    cursor_visible: bool,
    scroll_x: i32,
    scroll_y: i32
) {
    if !cursor_visible {
        return;
    }
    let cursor_width = 1;
    let cursor_height = font.height();

    let cursor_rect = Rect::new(
        cursor_x - scroll_x,
        cursor_y - scroll_y,
        cursor_width,
        cursor_height.try_into().unwrap(),
    );
    canvas.set_draw_color(Color::WHITE);
    canvas
        .fill_rect(cursor_rect)
        .expect("Failed to render cursor");
}

pub fn get_cursor_position(font: &Font, text: &str, cursor_index: usize) -> (i32, i32) {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut cursor_x = 0;
    let mut cursor_y = 0;
    let mut current_index = 0;

    for line in lines {
        if current_index + line.chars().count() >= cursor_index {
            let (left, _) = text.split_at(cursor_index - current_index);
            cursor_x = font.size_of(left).unwrap().0 as i32;
            break;
        }
        cursor_y += font.height() as i32;
        current_index += line.chars().count() + 1;
    }
    (cursor_x, cursor_y)
}

pub fn get_text_size(text: &str, font: &Font) -> (u32, u32) {
    let lines = text.lines();
    let mut text_width = 0;
    let mut text_height = 0;

    for line in lines {
        let (line_width, line_height) = font.size_of(line).unwrap();
        text_width = text_width.max(line_width);
        text_height += line_height
    }
    (text_width, text_height)
}