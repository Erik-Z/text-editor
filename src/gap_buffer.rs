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
        if new_cursor > self.length(){
            return;
        }

        if self.cursor < self.gap_start {
            self.cursor += 1;
        } else if self.cursor < self.buffer.len() - (self.gap_end - self.gap_start) {
            self.cursor += 1;
            self.gap_start += 1;
            self.buffer.swap(self.gap_start - 1, self.gap_end);
            self.gap_end += 1;
        }

        if new_cursor < self.cursor {
            self.buffer.copy_within(new_cursor..self.cursor, new_cursor + self.gap_end - self.gap_start);
            self.gap_end -= self.cursor - new_cursor;
            self.gap_start = new_cursor;
        } else if new_cursor > self.cursor {
            self.buffer.copy_within(self.cursor + self.gap_end - self.gap_start..new_cursor, self.cursor);
            self.gap_start += new_cursor - self.cursor;
            self.gap_end += new_cursor - self.cursor;
        }
        self.cursor = new_cursor;
    }

    fn resize(&mut self) {
        let new_capacity = self.buffer.len() * 2;
        let mut new_buffer = vec![' '; new_capacity];
        new_buffer[..self.gap_start].copy_from_slice(&self.buffer[..self.gap_start]);
        new_buffer[new_capacity - (self.buffer.len() - self.gap_end)..].copy_from_slice(&self.buffer[self.gap_end..]);

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
