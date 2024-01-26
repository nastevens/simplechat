/// Widget for getting and displaying user input
///
/// Adapted from the ratatui `user_input.rs` example

use ratatui::{
    prelude::{Buffer, Rect},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
};

/// Actions the TextInput understands
#[derive(Debug)]
pub enum TextInputAction {
    /// Adds a character
    Char(char),
    /// Moves cursor to the right
    MoveRight,
    /// Moves cursor to the left
    MoveLeft,
    /// Deletes character before cursor
    Backspace,
    /// Deletes character under cursor
    Delete,
    /// Clears input area
    Clear,
}

/// Simple text input widget
#[derive(Debug, Default)]
pub struct TextInput {
    cursor_position: usize,
    input: String,
}

impl Widget for TextInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf)
    }
}

impl Widget for &TextInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.input.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .padding(Padding::horizontal(1))
                    .title("Input"),
            )
            .render(area, buf)
    }
}

impl TextInput {
    /// Updates state based on `TextInputAction`
    pub fn action(&mut self, action: TextInputAction) {
        use TextInputAction::*;
        match action {
            Char(c) => self.enter_char(c),
            MoveRight => self.move_cursor_right(),
            MoveLeft => self.move_cursor_left(),
            Backspace => self.backspace(),
            Delete => self.delete(),
            Clear => self.clear(),
        }
    }

    /// Positions cursor properly given the `Rect` of this input box
    pub fn cursor_position(&self, area: Rect) -> (u16, u16) {
        (area.x + self.cursor_position as u16 + 2, area.y + 1)
    }

    /// Gets input collected so far
    pub fn get_input(&self) -> String {
        self.input.clone()
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);
        self.move_cursor_right();
    }

    fn backspace(&mut self) {
        if self.cursor_position != 0 {
            self.delete_at(self.cursor_position - 1);
            self.move_cursor_left();
        }
    }

    fn delete(&mut self) {
        self.delete_at(self.cursor_position);
    }

    fn delete_at(&mut self, idx: usize) {
        // Note that we can't use `remove` here because it removes bytes, not characters
        self.input = self
            .input
            .chars()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, c)| c)
            .collect();
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    fn clear(&mut self) {
        self.input.clear();
        self.reset_cursor();
    }
}
