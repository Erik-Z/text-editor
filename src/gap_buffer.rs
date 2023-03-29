pub struct GapBuffer {
    buffer: Vec<char>,
    gap_start: usize,
    gap_end: usize,
    cursor: usize,
}

impl GapBuffer {
    pub fn new(capacity: usize) -> Self {
        GapBuffer {
            buffer: vec![' '; capacity],
            gap_start: 0,
            gap_end: capacity,
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
            None
        } else {
            self.cursor -= 1;
            self.gap_start -= 1;
            let removed = self.buffer[self.cursor];
            self.buffer[self.cursor] = ' ';
            Some(removed)
        }
    }

    pub fn move_cursor(&mut self, new_cursor: usize) {
        if new_cursor > self.length() {
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
            self.buffer.copy_within(
                self.gap_end..(self.gap_end + shift_abs),
                self.gap_start,
            );
            self.gap_start += shift_abs;
            self.gap_end += shift_abs;
            self.cursor += shift_abs;
        }
    }

    fn resize(&mut self) {
        let new_capacity = self.buffer.len() * 2;
        let mut new_buffer = vec![' '; new_capacity];
        new_buffer[..self.gap_start].copy_from_slice(&self.buffer[..self.gap_start]);
        new_buffer[new_capacity - (self.buffer.len() - self.gap_end)..]
            .copy_from_slice(&self.buffer[self.gap_end..]);

        let gap_size = new_capacity - self.buffer.len() + self.gap_end - self.gap_start;
        self.gap_end = self.gap_start + gap_size;
        self.buffer = new_buffer;
    }

    pub fn length(&self) -> usize {
        self.buffer.len() - (self.gap_end - self.gap_start)
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }

    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.buffer.len() - (self.gap_end - self.gap_start));
        result.extend(self.buffer[..self.gap_start].iter());
        result.extend(self.buffer[self.gap_end..].iter());
        result
    }
}
