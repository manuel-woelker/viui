// Module for editing text state

use crate::nodes::events::{InputEvent, InputEventKind, KeyboardKey};
use std::fmt::Display;

pub struct TextEditState<'a> {
    //    pub selection: Range<usize>,
    pub text: &'a str,
    pub cursor_position: usize,
    pub new_text: Option<String>,
}

impl<'a> TextEditState<'a> {
    pub fn new(text: &'a str, cursor_position: usize) -> Self {
        let state = Self {
            text,
            cursor_position,
            new_text: None,
        };
        state.assert_char_boundary();
        state
    }

    pub fn handle_event(&mut self, event: &InputEvent) {
        match event.kind() {
            InputEventKind::Character(character) => {
                if !character.is_control() {
                    let mut new_text = self.text.to_string();
                    new_text.insert(self.cursor_position, *character);
                    self.cursor_position += character.len_utf8();
                    self.new_text = Some(new_text);
                }
            }
            InputEventKind::KeyInput(key) => match key {
                KeyboardKey::ArrowLeft => {
                    if self.cursor_position > 0 {
                        let len = self.text[..self.cursor_position]
                            .chars().next_back()
                            .unwrap()
                            .len_utf8();
                        self.cursor_position -= len;
                    }
                }
                KeyboardKey::ArrowRight => {
                    if self.cursor_position < self.text.len() {
                        let len = self.text[self.cursor_position..]
                            .chars()
                            .next()
                            .unwrap()
                            .len_utf8();
                        self.cursor_position += len;
                    }
                }
                KeyboardKey::Home => {
                    self.cursor_position = 0;
                }
                KeyboardKey::End => {
                    self.cursor_position = self.text.len();
                }
                KeyboardKey::Delete => {
                    if self.cursor_position < self.text.len() {
                        let mut new_text = self.text.to_string();
                        new_text.remove(self.cursor_position);
                        self.new_text = Some(new_text);
                    }
                }
                KeyboardKey::Backspace => {
                    if self.cursor_position > 0 {
                        let mut new_text = self.text.to_string();
                        let len = self.text[..self.cursor_position]
                            .chars().next_back()
                            .unwrap()
                            .len_utf8();
                        self.cursor_position -= len;
                        new_text.remove(self.cursor_position);
                        self.new_text = Some(new_text);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        self.assert_char_boundary();
    }

    fn assert_char_boundary(&self) {
        // Cursor position must always be on a char boundary
        assert!(self
            .new_text
            .as_deref()
            .unwrap_or(self.text)
            .is_char_boundary(self.cursor_position));
    }
}

impl Display for TextEditState<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut string = self
            .new_text
            .clone()
            .unwrap_or_else(|| self.text.to_string());
        let cursor_position = self.cursor_position.min(string.len());
        if string.is_char_boundary(self.cursor_position) {
            string.insert(cursor_position, '|');
        } else {
            string += format!(" INVALID CURSOR POSITION: {}", cursor_position).as_str();
        }
        f.write_str(&string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::events::{InputEvent, KeyboardKey};

    #[test]
    fn test_display() {
        let mut text_state = TextEditState::new("123", 0);
        assert_eq!(text_state.to_string(), "|123");
        text_state.cursor_position = 1;
        assert_eq!(text_state.to_string(), "1|23");
        text_state.cursor_position = 2;
        assert_eq!(text_state.to_string(), "12|3");
        text_state.cursor_position = 3;
        assert_eq!(text_state.to_string(), "123|");
    }

    #[test]
    fn test_display_multibyte_chars() {
        let mut text_state = TextEditState::new("€", 0);
        assert_eq!(text_state.to_string(), "|€");
        text_state.cursor_position = 1;
        assert_eq!(text_state.to_string(), "€ INVALID CURSOR POSITION: 1");
        text_state.cursor_position = 3;
        assert_eq!(text_state.to_string(), "€|");
    }

    fn test_handle_event(
        initial_text: &str,
        cursor_position: usize,
        event: InputEvent,
        expected: &str,
    ) {
        let mut text_state = TextEditState::new(initial_text, cursor_position);
        text_state.handle_event(&event);
        assert_eq!(text_state.to_string(), expected);
    }

    macro_rules! test_events {
        ($($name: ident, $initial_text: expr, $cursor_position: expr, $event: expr, $expected: expr;)*) => {
            $(
                #[test]
                fn $name() {
                    test_handle_event($initial_text, $cursor_position, $event, $expected);
                }

            )*
        };
    }

    test_events!(
        // Characters
        test_insert_char_0, "123", 0, InputEvent::character('x'), "x|123";
        test_insert_char_1, "123", 1, InputEvent::character('x'), "1x|23";
        test_insert_char_3, "123", 3, InputEvent::character('x'), "123x|";

        // Left
        test_left_empty, "", 0, InputEvent::key_input(KeyboardKey::ArrowLeft), "|";
        test_left_0, "123", 0, InputEvent::key_input(KeyboardKey::ArrowLeft), "|123";
        test_left_1, "123", 1, InputEvent::key_input(KeyboardKey::ArrowLeft), "|123";
        test_left_2, "123", 2, InputEvent::key_input(KeyboardKey::ArrowLeft), "1|23";
        test_left_multibyte, "€", 3, InputEvent::key_input(KeyboardKey::ArrowLeft), "|€";

        // Right
        test_right_empty, "", 0, InputEvent::key_input(KeyboardKey::ArrowRight), "|";
        test_right_0, "123", 0, InputEvent::key_input(KeyboardKey::ArrowRight), "1|23";
        test_right_1, "123", 1, InputEvent::key_input(KeyboardKey::ArrowRight), "12|3";
        test_right_3, "123", 3, InputEvent::key_input(KeyboardKey::ArrowRight), "123|";
        test_right_multibyte, "€", 0, InputEvent::key_input(KeyboardKey::ArrowRight), "€|";

        // Home key
        test_home_empty, "", 0, InputEvent::key_input(KeyboardKey::Home), "|";
        test_home_0, "123", 0, InputEvent::key_input(KeyboardKey::Home), "|123";
        test_home_1, "123", 1, InputEvent::key_input(KeyboardKey::Home), "|123";
        test_home_3, "123", 3, InputEvent::key_input(KeyboardKey::Home), "|123";

        // End key
        test_end_empty, "", 0, InputEvent::key_input(KeyboardKey::End), "|";
        test_end_0, "123", 0, InputEvent::key_input(KeyboardKey::End), "123|";
        test_end_1, "123", 1, InputEvent::key_input(KeyboardKey::End), "123|";
        test_end_3, "123", 3, InputEvent::key_input(KeyboardKey::End), "123|";

        // Delete key
        test_delete_empty, "", 0, InputEvent::key_input(KeyboardKey::Delete), "|";
        test_delete_0, "123", 0, InputEvent::key_input(KeyboardKey::Delete), "|23";
        test_delete_1, "123", 1, InputEvent::key_input(KeyboardKey::Delete), "1|3";
        test_delete_2, "123", 2, InputEvent::key_input(KeyboardKey::Delete), "12|";
        test_delete_3, "123", 3, InputEvent::key_input(KeyboardKey::Delete), "123|";
        test_delete_multibyte, "€ß", 0, InputEvent::key_input(KeyboardKey::Delete), "|ß";

        // Backspace key
        test_backspace_empty, "", 0, InputEvent::key_input(KeyboardKey::Backspace), "|";
        test_backspace_0, "123", 0, InputEvent::key_input(KeyboardKey::Backspace), "|123";
        test_backspace_1, "123", 1, InputEvent::key_input(KeyboardKey::Backspace), "|23";
        test_backspace_2, "123", 2, InputEvent::key_input(KeyboardKey::Backspace), "1|3";
        test_backspace_3, "123", 3, InputEvent::key_input(KeyboardKey::Backspace), "12|";
        test_backspace_multibyte, "€ß", 3, InputEvent::key_input(KeyboardKey::Backspace), "|ß";
    );
}
