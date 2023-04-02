use sdl2::{
    self,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureQuery},
    ttf::Font,
    video::{Window, WindowContext},
};

use crate::{gap_buffer::GapBuffer, settings};

pub fn render_text(
    canvas: &mut Canvas<Window>,
    font: &Font,
    text: &str,
    scroll_x: i32,
    scroll_y: i32,
) {
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
    scroll_y: i32,
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

pub fn render_scroll_bars(
    canvas: &mut Canvas<Window>,
    (window_width, window_height): (u32, u32),
    (scroll_bar_width, scroll_bar_height): (u32, u32),
    (text_width, text_height): (u32, u32),
    (scroll_x, scroll_y): (i32, i32),
    (max_scroll_x, max_scroll_y): (u32, u32),
) -> (Rect, Rect) {
    let vertical_scroll_bar = Rect::new(
        window_width as i32,
        0,
        scroll_bar_width,
        window_height,
    );
    let horizontal_scroll_bar = Rect::new(
        0,
        window_height as i32,
        window_width,
        scroll_bar_height,
    );

    let vertical_handle_height =
        ((window_height as f32 / text_height as f32) * window_height as f32) as u32;
    let horizontal_handle_width =
        ((window_width as f32 / text_width as f32) * window_width as f32) as u32;

    let vertical_handle_y = (scroll_y as f32 / max_scroll_y as f32
        * ((window_height).saturating_sub(vertical_handle_height)) as f32)
        as i32;
    let horizontal_handle_x = (scroll_x as f32 / max_scroll_x as f32
        * ((window_width).saturating_sub(horizontal_handle_width)) as f32)
        as i32;

    let vertical_handle = Rect::new(
        window_width as i32,
        vertical_handle_y,
        scroll_bar_width,
        vertical_handle_height,
    );
    let horizontal_handle = Rect::new(
        horizontal_handle_x,
        window_height as i32,
        horizontal_handle_width,
        scroll_bar_height,
    );

    canvas.set_draw_color(Color::RGB(200, 200, 200));
    canvas.fill_rect(vertical_scroll_bar).unwrap();
    canvas.fill_rect(horizontal_scroll_bar).unwrap();
    canvas.set_draw_color(Color::RGB(100, 100, 100));
    canvas.fill_rect(vertical_handle).unwrap();
    canvas.fill_rect(horizontal_handle).unwrap();

    (vertical_handle, horizontal_handle)
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

pub fn get_nearest_character_position(font: &Font, text: &str, x: i32, y: i32) -> usize {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut nearest_char_index = 0;
    let mut current_index = 0;

    let line_height = font.height() as i32;

    // Find the nearest line based on the y-coordinate
    let line_index = (y / line_height).clamp(0, lines.len() as i32 - 1) as usize;
    let line = lines[line_index];

    for (char_index, _) in line.char_indices() {
        let (left, _) = line.split_at(char_index);
        let char_x = font.size_of(left).unwrap().0 as i32;

        if char_x > x {
            nearest_char_index = current_index + char_index;
            break;
        }
    }

    // If the click was beyond the last character in the line, move the cursor to the end of the line
    if nearest_char_index == 0 {
        nearest_char_index = current_index + line.chars().count();
    }

    for idx in 0..line_index {
        current_index += lines[idx].chars().count() + 1;
    }

    nearest_char_index += current_index;
    nearest_char_index
}

