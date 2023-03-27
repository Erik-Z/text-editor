mod constants;
mod gap_buffer;
use sdl2::{
    self,
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureQuery, TextureCreator},
    ttf::{Sdl2TtfContext, Font},
    video::{Window, WindowContext},
};
use std::time::Duration;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to initialize video subsystem/");
    let ttf_context = sdl2::ttf::init().expect("Failed to initialize ttf context.");

    let window = video_subsystem
        .window("Text Editor", 800, 600)
        .position_centered()
        .build()
        .expect("Failed to build window");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("Failed to build canvas");

    let font_size = 20;
    let font = ttf_context
        .load_font(constants::FONT_PATH, font_size)
        .expect("Failed to load font.");

    let mut buffer = gap_buffer::GapBuffer::new(1024);
    let mut cursor = 0;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    if buffer.length() > 0 {
                        buffer.insert('\n');
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => {
                    buffer.remove();
                }
                Event::TextInput {
                    timestamp: _,
                    window_id: _,
                    text,
                } => {
                    buffer.insert(text.chars().next().unwrap());
                    // println!("{}", text);
                }
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        render_text(
            &mut canvas,
            &font,
            &buffer.to_string(),
        );

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn render_text(
    canvas: &mut Canvas<Window>,
    font: &Font,
    text: &str,
) {
    let text_surface;
    if text.len() == 0 {
        text_surface = font
            .render("Hello\r\nWorld!")
            .blended(Color::WHITE)
            .expect("Failed to render font.");
    } else {
        text_surface = font
            .render(text)
            .blended(Color::WHITE)
            .expect("Failed to render font.");
    }

    let texture_creator = canvas.texture_creator();
    let text_texture = texture_creator
        .create_texture_from_surface(&text_surface)
        .unwrap();

    let TextureQuery { width, height, .. } = text_texture.query();

    let x = (0) / 2;
    let y = (0) / 2;

    let dst = Rect::new(x as i32, y as i32, width, height);
    canvas.copy(&text_texture, None, Some(dst)).unwrap();
}