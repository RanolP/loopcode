use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};

use crate::app::{InputEvent, KeyInput};

pub(crate) fn map_input_event(event: Event) -> Option<InputEvent> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            let word_modifier = key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER);
            let submit_modifier = key.modifiers.contains(KeyModifiers::ALT);
            let ctrl_j_submit = key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char('j' | 'J'))
                && !is_vscode_terminal();
            match key.code {
                KeyCode::BackTab => Some(InputEvent::Key(KeyInput::ShiftTab)),
                KeyCode::Left if word_modifier => Some(InputEvent::Key(KeyInput::WordLeft)),
                KeyCode::Right if word_modifier => Some(InputEvent::Key(KeyInput::WordRight)),
                KeyCode::Left => Some(InputEvent::Key(KeyInput::Left)),
                KeyCode::Right => Some(InputEvent::Key(KeyInput::Right)),
                KeyCode::Up => Some(InputEvent::Key(KeyInput::Up)),
                KeyCode::Down => Some(InputEvent::Key(KeyInput::Down)),
                KeyCode::Home => Some(InputEvent::Key(KeyInput::Home)),
                KeyCode::End => Some(InputEvent::Key(KeyInput::End)),
                KeyCode::Backspace if word_modifier => {
                    Some(InputEvent::Key(KeyInput::BackspaceWord))
                }
                KeyCode::Backspace => Some(InputEvent::Key(KeyInput::Backspace)),
                KeyCode::Char('w' | 'W') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(InputEvent::Key(KeyInput::BackspaceWord))
                }
                KeyCode::Delete => Some(InputEvent::Key(KeyInput::Delete)),
                _ if ctrl_j_submit => Some(InputEvent::Key(KeyInput::Submit)),
                KeyCode::Enter if submit_modifier => Some(InputEvent::Key(KeyInput::Submit)),
                KeyCode::Enter => Some(InputEvent::Key(KeyInput::Enter)),
                KeyCode::Char('c' | 'C') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(InputEvent::Key(KeyInput::Interrupt))
                }
                KeyCode::Esc => Some(InputEvent::Key(KeyInput::Esc)),
                KeyCode::Char(ch) => Some(InputEvent::Key(KeyInput::Char(ch))),
                _ => None,
            }
        }
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::ScrollUp => Some(InputEvent::ScrollLines(-1)),
            MouseEventKind::ScrollDown => Some(InputEvent::ScrollLines(1)),
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => Some(
                InputEvent::MouseDown {
                    x: mouse.column,
                    y: mouse.row,
                },
            ),
            _ => None,
        },
        _ => None,
    }
}

fn is_vscode_terminal() -> bool {
    std::env::var("TERM_PROGRAM")
        .map(|v| v.eq_ignore_ascii_case("vscode"))
        .unwrap_or(false)
}
