mod constants;
mod event_handling;
mod gap_buffer;
mod rendering;
mod settings;
use rendering::{
    get_cursor_position, get_text_size, render_cursor, render_scroll_bars, render_text, get_nearest_character_position,
};
use sdl2::{
    self,
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::Color,
    rect::{Rect, Point},
};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use settings::{WINDOW_WIDTH, WINDOW_HEIGHT};
use std::fs::File;
use std::io::Read;

use std::time::{Duration, Instant};

//TODO: Implement Delete Method
//TODO: Implement Insert Mode
//TODO: Implement Copy and Paste
//TODO: Implement loading and saving a file.
//TODO: Check if contents are connected to a file path
//TODO: Check if there are any changes made.
//TODO: Click to move cursor
//TODO: Implement changing font size

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to initialize video subsystem/");
    let ttf_context = sdl2::ttf::init().expect("Failed to initialize ttf context.");

    let window = video_subsystem
        .window("Text Editor", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        // .resizable()
        .build()
        .expect("Failed to build window");

    let (mut window_width, mut window_height) = window.size();
    (window_width, window_height) = (window_width - settings::SCROLL_BAR_WIDTH, window_height - settings::SCROLL_BAR_HEIGHT);

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("Failed to build canvas");

    let font = ttf_context
        .load_font(constants::FONT_PATH, settings::font_size)
        .expect("Failed to load font.");

    let mut buffer = gap_buffer::GapBuffer::new(1024);

    let mut viewport_width = WINDOW_WIDTH;
    let mut viewport_height = WINDOW_HEIGHT;
    let mut viewport = Rect::new(0, 0, viewport_width, viewport_height);
    canvas.set_viewport(Some(viewport));

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut file_path_for_content = None;
    let mut has_file_been_saved = false;
    let mut file_name = String::new();;

    let mut cursor_visible = true;
    let mut last_cursor_blink = Instant::now();

    let mut scroll_x: i32 = 0;
    let mut scroll_y: i32 = 0;
    let mut max_scroll_x = 0;
    let mut max_scroll_y = 0;
    let mut can_scroll_vertically_up = false;
    let mut can_scroll_vertically_down = false;
    let mut can_scroll_horizontally = false;

    let mut vertical_scroll_bar = Rect::new(0,0,0,0);
    let mut horizontal_scroll_bar = Rect::new(0,0,0,0);
    let mut vertical_handle_height = 0;
    let mut horizontal_handle_width = 0;
    let mut dragging_scroll_bar_vertical = false;
    let mut dragging_scroll_bar_horizontal = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(w, h) => {
                        window_width = w as u32 - settings::SCROLL_BAR_WIDTH;
                        window_height = h as u32 - settings::SCROLL_BAR_HEIGHT;
                        viewport_width = w as u32 - settings::SCROLL_BAR_WIDTH;
                        viewport_height = h as u32 - settings::SCROLL_BAR_HEIGHT;
                        viewport = Rect::new(0, 0, viewport_width, viewport_height);
                        canvas.set_viewport(Some(viewport));
                    }
                    _ => {}
                },
                Event::MouseButtonDown { x, y, .. } => {
                    if vertical_scroll_bar.contains_point(Point::new(x, y)) {
                        dragging_scroll_bar_vertical = true;
                    } else if horizontal_scroll_bar.contains_point(Point::new(x, y)) {
                        dragging_scroll_bar_horizontal = true;
                    } else {
                        let cursor_index = get_nearest_character_position(&font, &buffer.to_string(), x + scroll_x, y + scroll_y);
                        buffer.move_cursor(cursor_index);
                    }
                }
                Event::MouseButtonUp { .. } => {
                    dragging_scroll_bar_vertical = false;
                    dragging_scroll_bar_horizontal = false;
                }
                Event::MouseMotion { x, y, .. } => {
                    if dragging_scroll_bar_vertical {
                        let new_handle_y = y - vertical_scroll_bar.height() as i32 / 2;
                        let new_scroll_y = (new_handle_y as f32 / (window_height - vertical_handle_height) as f32 * max_scroll_y as f32) as i32;
                        scroll_y = new_scroll_y.clamp(0, max_scroll_y as i32);
                    } else if dragging_scroll_bar_horizontal {
                        let new_handle_x = x - horizontal_scroll_bar.width() as i32 / 2;
                        let new_scroll_x = (new_handle_x as f32 / (window_width - horizontal_handle_width) as f32 * max_scroll_x as f32) as i32;
                        scroll_x = new_scroll_x.clamp(0, max_scroll_x as i32);
                    }
                }
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
                    has_file_been_saved = false;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => {
                    buffer.remove();
                    has_file_been_saved = false;
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
                    has_file_been_saved = false;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod,
                    ..
                } => {
                    if keycode == Keycode::O && keymod.contains(sdl2::keyboard::Mod::LCTRLMOD) {
                        file_path_for_content = FileDialog::new()
                            .set_location("~/Desktop")
                            .add_filter("Text Documents", &["txt"])
                            .add_filter("All Documents", &["*"])
                            .show_open_single_file()
                            .expect("Failed to get file path.");

                        if let Some(path) = file_path_for_content {
                            let mut file = match File::open(&path) {
                                Ok(file) => file,
                                Err(e) => {
                                    eprintln!("Unable to open file: {:?}", e);
                                    continue;
                                }
                            };
                            let mut contents = String::new();
                            if let Err(e) = file.read_to_string(&mut contents) {
                                eprintln!("Unable to read file: {:?}", e);
                                continue;
                            }

                            file_name = path
                                .file_name()
                                .and_then(|os_str| os_str.to_str())
                                .map(|str| str.to_string()).unwrap();


                            // Clear the buffer and insert the file's contents
                            buffer.clear();
                            for c in contents.chars() {
                                buffer.insert(c);
                            }
                            has_file_been_saved = true;
                        }
                    }
                }
                Event::MouseWheel { mut y, .. } => {
                    if !can_scroll_vertically_up {
                        if y > 0 {
                            y = 0
                        }
                    }
                    if !can_scroll_vertically_down {
                        if y < 0 {
                            y = 0
                        }
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
                    has_file_been_saved = false;
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

        if file_name == String::new(){
            canvas.window_mut().set_title(&format!("{}{}", &"Untitled", if has_file_been_saved {&""} else {&"*"})).unwrap();
        } else {
            canvas.window_mut().set_title(&format!("{}{}", &file_name, if has_file_been_saved {&""} else {&"*"})).unwrap();
        }
        

        render_text(&mut canvas, &font, &buffer.to_string(), scroll_x, scroll_y);

        let (cursor_x, cursor_y) =
            get_cursor_position(&font, &buffer.to_string(), buffer.get_cursor());

        render_cursor(
            &mut canvas,
            &font,
            cursor_x,
            cursor_y,
            cursor_visible,
            scroll_x,
            scroll_y,
        );

        let (text_width, text_height) = get_text_size(&buffer.to_string(), &font);

        max_scroll_x = if text_width > window_width {
            text_width - window_width
        } else {
            0
        };
        max_scroll_y = if text_height > window_height {
            text_height - window_height
        } else {
            0
        };

        vertical_handle_height =
        ((window_height as f32 / text_height as f32) * window_height as f32) as u32;
        horizontal_handle_width =
            ((window_width as f32 / text_width as f32) * window_width as f32) as u32;

        (vertical_scroll_bar, horizontal_scroll_bar) = render_scroll_bars(
            &mut canvas,
            (window_width, window_height),
            (settings::SCROLL_BAR_WIDTH, settings::SCROLL_BAR_HEIGHT),
            (text_width, text_height),
            (scroll_x, scroll_y),
            (max_scroll_x, max_scroll_y),
        );

        can_scroll_vertically_down =
            text_height > viewport_height && scroll_y < max_scroll_y as i32;
        can_scroll_vertically_up = text_height > viewport_height && scroll_y > 0;
        can_scroll_horizontally = text_width > viewport_width && scroll_x < max_scroll_x as i32;

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
