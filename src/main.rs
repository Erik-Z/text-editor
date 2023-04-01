mod constants;
mod gap_buffer;
mod rendering;
mod settings;
use rendering::{get_cursor_position, render_cursor, render_text, get_text_size};
use sdl2::{self, event::{Event, WindowEvent}, keyboard::Keycode, pixels::Color, rect::Rect};
use std::time::{Duration, Instant};

//TODO: Implement Delete Method
//TODO: Implement Insert Mode
//TODO: Implement Copy and Paste
//TODO: Implement loading and saving a file.
//TODO: Implement changing font size

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to initialize video subsystem/");
    let ttf_context = sdl2::ttf::init().expect("Failed to initialize ttf context.");

    let window = video_subsystem
        .window("Text Editor", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .expect("Failed to build window");

    let (mut window_width, mut window_height) = window.size();

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("Failed to build canvas");
    
    let font = ttf_context
        .load_font(constants::FONT_PATH, settings::font_size)
        .expect("Failed to load font.");

    let mut buffer = gap_buffer::GapBuffer::new(1024);

    let mut viewport_width = 800;
    let mut viewport_height = 600;
    let mut viewport = Rect::new(0, 0, viewport_width, viewport_height);
    canvas.set_viewport(Some(viewport));

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut cursor_visible = true;
    let mut last_cursor_blink = Instant::now();
    let mut scroll_x: i32 = 0;
    let mut scroll_y: i32 = 0;
    let mut can_scroll_vertically_up = false;
    let mut can_scroll_vertically_down = false;
    let mut can_scroll_horizontally = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(w, h) => {
                        window_width = w as u32;
                        window_height = h as u32;
                        viewport_width = w as u32;
                        viewport_height = h as u32;
                        viewport = Rect::new(0, 0, viewport_width, viewport_height);
                        canvas.set_viewport(Some(viewport));
                    }
                    _ => {}
                },
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
                Event::KeyDown {
                    keycode: Some(Keycode::Delete),
                    ..
                } => {
                    buffer.remove();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    if buffer.get_cursor() != 0 {
                        buffer.move_cursor(buffer.get_cursor() - 1);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    buffer.move_cursor(buffer.get_cursor() + 1);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    let (row, col) = buffer.get_cursor_position();
                    if row > 0 {
                        // Move the cursor up a line
                        let target_row = row - 1;
                        let mut new_cursor = 0;
                        let mut current_row = 0;
                        let mut current_col = 0;

                        for (i, c) in buffer.to_string().chars().enumerate() {
                            if current_row == target_row && (current_col == col || c == '\n') {
                                new_cursor = i;
                                break;
                            }
                            if c == '\n' {
                                current_row += 1;
                                current_col = 0;
                            } else {
                                current_col += 1;
                            }
                        }

                        buffer.move_cursor(new_cursor);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    let (row, col) = buffer.get_cursor_position();
                    let total_rows = buffer.to_string().lines().count();
                    if row < total_rows - 1 {
                        // Move the cursor down a line
                        let target_row = row + 1;
                        let mut new_cursor = 0;
                        let mut current_row = 0;
                        let mut current_col = 0;

                        for (i, c) in buffer.to_string().chars().enumerate() {
                            if current_row == target_row {
                                if current_col == col || c == '\n' || c == constants::EOF_CHAR {
                                    new_cursor = i;
                                    break;
                                }
                            }
                            if c == '\n' {
                                current_row += 1;
                                current_col = 0;
                            } else {
                                current_col += 1;
                            }
                        }

                        buffer.move_cursor(new_cursor);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Tab),
                    ..
                } => {
                    for _ in 0..settings::tab_width {
                        buffer.insert(' ');
                    }
                }
                Event::MouseWheel { mut y, .. } => {
                    if !can_scroll_vertically_up {
                        if y > 0 { y = 0 }    
                    } 
                    if !can_scroll_vertically_down {
                        if y < 0 { y = 0 }
                    }
                    scroll_y += y * -10;
                    if scroll_y < 0 {
                        scroll_y = 0;
                    }
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

        // Keeps track of cursor blinking
        if last_cursor_blink.elapsed() >= Duration::from_millis(constants::CURSOR_BLINK_DURATION) {
            cursor_visible = !cursor_visible;
            last_cursor_blink = Instant::now();
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        render_text(&mut canvas, &font, &buffer.to_string(), scroll_x, scroll_y);

        let (cursor_x, cursor_y) =
            get_cursor_position(&font, &buffer.to_string(), buffer.get_cursor());
        render_cursor(&mut canvas, &font, cursor_x, cursor_y, cursor_visible, scroll_x, scroll_y);

        let (text_width, text_height) = get_text_size(&buffer.to_string(), &font);

        let max_scroll_x = if text_width > window_width {
            text_width - window_width
        } else {
            0
        };
        let max_scroll_y = if text_height > window_height {
            text_height - window_height
        } else {
            0
        };

        can_scroll_vertically_down = text_height > viewport_height && scroll_y < max_scroll_y as i32;
        can_scroll_vertically_up = text_height > viewport_height && scroll_y > 0;
        can_scroll_horizontally =  text_width > viewport_width && scroll_x < max_scroll_x as i32;

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
