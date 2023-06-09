use crate::constants::EOF_CHAR;

pub struct GapBuffer {
    buffer: Vec<char>,
    gap_start: usize,
    gap_end: usize,
    cursor: usize,
}

impl GapBuffer {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = vec![' '; capacity];
        buffer[capacity - 1] = EOF_CHAR;

        GapBuffer {
            buffer,
            gap_start: 0,
            gap_end: capacity - 1,
            cursor: 0,
        }
    }

    pub fn insert(&mut self, ch: char) {
        if self.gap_start == self.gap_end {
            self.resize();
        }

        self.buffer[self.gap_start] = ch;
        self.gap_start += 1;
        self.cursor += 1;
    }

    pub fn remove(&mut self) -> Option<char> {
        if self.cursor == 0 {
            return None;
        } else {
            self.cursor -= 1;
            self.gap_start -= 1;
            let removed = self.buffer[self.cursor];
            self.buffer[self.cursor] = ' ';
            Some(removed)
        }
    }

    pub fn move_cursor(&mut self, new_cursor: usize) {
        if new_cursor > self.length() - 1 {
            return;
        }

        if new_cursor == self.cursor {
            return;
        }

        let shift = (new_cursor as isize) - (self.cursor as isize);

        if shift < 0 {
            // Shift the cursor to the left
            let shift_abs = shift.abs() as usize;
            self.buffer.copy_within(
                (self.gap_start - shift_abs)..self.gap_start,
                self.gap_end - shift_abs,
            );
            self.gap_start -= shift_abs;
            self.gap_end -= shift_abs;
            self.cursor -= shift_abs;
        } else if shift > 0 {
            // Shift the cursor to the right
            let shift_abs = shift as usize;
            self.buffer
                .copy_within(self.gap_end..(self.gap_end + shift_abs), self.gap_start);
            self.gap_start += shift_abs;
            self.gap_end += shift_abs;
            self.cursor += shift_abs;
        }
    }

    fn resize(&mut self) {
        let new_capacity = self.buffer.len() * 2;
        let mut new_buffer = vec![' '; new_capacity];
        new_buffer[new_capacity - 1] = EOF_CHAR;
        new_buffer[..self.gap_start].copy_from_slice(&self.buffer[..self.gap_start]);
        new_buffer[new_capacity - (self.buffer.len() - self.gap_end)..]
            .copy_from_slice(&self.buffer[self.gap_end..]);

        let gap_size = new_capacity - self.buffer.len() + self.gap_end - self.gap_start;
        self.gap_end = self.gap_start + gap_size;
        self.buffer = new_buffer;
    }

    pub fn clear(&mut self) {
        self.gap_start = 0;
        self.gap_end = self.buffer.len() - 1;
        self.cursor = 0;

        let buffer_len = self.buffer.len();
        for i in 0..buffer_len - 1 {
            self.buffer[i] = ' ';
        }
        self.buffer[buffer_len - 1] = EOF_CHAR;
    }

    pub fn length(&self) -> usize {
        self.buffer.len() - (self.gap_end - self.gap_start)
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }

    pub fn get_cursor_position(&self) -> (usize, usize) {
        let mut row = 0;
        let mut col = 0;

        for (i, c) in self.to_string().chars().enumerate() {
            if i == self.get_cursor() {
                break;
            }
            if c == '\n' {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        (row, col)
    }

    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.buffer.len() - (self.gap_end - self.gap_start));
        result.extend(self.buffer[..self.gap_start].iter());
        result.extend(self.buffer[self.gap_end..].iter());
        result
    }
}
