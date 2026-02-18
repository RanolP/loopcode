use crate::{FocusId, Node};
use std::collections::HashMap;

pub trait UiApp {
    fn render(&mut self) -> Node;

    fn on_input(&mut self, _event: UiInputEvent) {}

    fn set_window_size(&mut self, _size: WindowSize) {}

    fn focus_state(&mut self) -> Option<&mut FocusState> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiKeyInput {
    Tab,
    BackTab,
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Backspace,
    BackspaceWord,
    Delete,
    Enter,
    Submit,
    Esc,
    Char(char),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiInputEvent {
    Key(UiKeyInput),
    ScrollLines(i16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusNavOutcome {
    Ignored,
    Handled,
    RequestQuit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusKind {
    Generic,
    TextInput,
    ScrollRegion,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FocusPath(pub Vec<usize>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusEntry {
    pub id: FocusId,
    pub path: FocusPath,
    pub kind: FocusKind,
}

#[derive(Clone, Debug, Default)]
pub struct TextInputState {
    value: String,
    cursor: usize,
    preferred_column: Option<usize>,
}

impl TextInputState {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.chars().count();
        Self {
            value,
            cursor,
            preferred_column: None,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.cursor.min(self.value.chars().count());
        self.preferred_column = None;
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor.min(self.value.chars().count());
        self.preferred_column = None;
    }

    pub fn handle_input(&mut self, event: UiInputEvent) -> bool {
        let UiInputEvent::Key(key) = event else {
            return false;
        };

        match key {
            UiKeyInput::Left => {
                self.cursor = self.cursor.saturating_sub(1);
                self.preferred_column = None;
                true
            }
            UiKeyInput::Right => {
                self.cursor = (self.cursor + 1).min(self.value.chars().count());
                self.preferred_column = None;
                true
            }
            UiKeyInput::WordLeft => {
                self.cursor = prev_word_boundary(&self.value, self.cursor);
                true
            }
            UiKeyInput::WordRight => {
                self.cursor = next_word_boundary(&self.value, self.cursor);
                true
            }
            UiKeyInput::Home => {
                self.cursor = 0;
                self.preferred_column = None;
                true
            }
            UiKeyInput::End => {
                self.cursor = self.value.chars().count();
                self.preferred_column = None;
                true
            }
            UiKeyInput::Up => {
                self.move_vertical(-1);
                true
            }
            UiKeyInput::Down => {
                self.move_vertical(1);
                true
            }
            UiKeyInput::BackspaceWord => {
                if self.cursor == 0 {
                    return false;
                }
                let start_char = prev_word_boundary(&self.value, self.cursor);
                let start = char_to_byte_index(&self.value, start_char);
                let end = char_to_byte_index(&self.value, self.cursor);
                self.value.replace_range(start..end, "");
                self.cursor = start_char;
                self.preferred_column = None;
                true
            }
            UiKeyInput::Backspace => {
                if self.cursor == 0 {
                    return false;
                }
                let end = char_to_byte_index(&self.value, self.cursor);
                let start = char_to_byte_index(&self.value, self.cursor - 1);
                self.value.replace_range(start..end, "");
                self.cursor -= 1;
                self.preferred_column = None;
                true
            }
            UiKeyInput::Delete => {
                let len = self.value.chars().count();
                if self.cursor >= len {
                    return false;
                }
                let start = char_to_byte_index(&self.value, self.cursor);
                let end = char_to_byte_index(&self.value, self.cursor + 1);
                self.value.replace_range(start..end, "");
                self.preferred_column = None;
                true
            }
            UiKeyInput::Char(ch) => {
                let idx = char_to_byte_index(&self.value, self.cursor);
                self.value.insert(idx, ch);
                self.cursor += 1;
                self.preferred_column = None;
                true
            }
            UiKeyInput::Enter => {
                let idx = char_to_byte_index(&self.value, self.cursor);
                self.value.insert(idx, '\n');
                self.cursor += 1;
                self.preferred_column = None;
                true
            }
            _ => false,
        }
    }

    fn move_vertical(&mut self, delta: i32) {
        let (line, col) = line_col_for_cursor(&self.value, self.cursor);
        let target_line = (line as i32 + delta).clamp(0, line_count(&self.value) as i32 - 1);
        let preferred = self.preferred_column.unwrap_or(col);
        self.cursor = cursor_for_line_col(&self.value, target_line as usize, preferred);
        self.preferred_column = Some(preferred);
    }
}

fn char_to_byte_index(value: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }
    value
        .char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len())
}

fn line_count(value: &str) -> usize {
    value.chars().filter(|ch| *ch == '\n').count() + 1
}

fn line_col_for_cursor(value: &str, cursor: usize) -> (usize, usize) {
    let mut line = 0usize;
    let mut col = 0usize;
    for (i, ch) in value.chars().enumerate() {
        if i == cursor {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn cursor_for_line_col(value: &str, target_line: usize, target_col: usize) -> usize {
    let mut line = 0usize;
    let mut col = 0usize;
    let mut idx = 0usize;

    for ch in value.chars() {
        if line == target_line && col >= target_col {
            break;
        }

        if ch == '\n' {
            if line == target_line {
                break;
            }
            line += 1;
            col = 0;
            idx += 1;
            continue;
        }

        if line == target_line {
            col += 1;
        }
        idx += 1;
    }

    idx
}

fn prev_word_boundary(value: &str, cursor: usize) -> usize {
    let chars: Vec<char> = value.chars().collect();
    let mut i = cursor.min(chars.len());

    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }
    while i > 0 && !chars[i - 1].is_whitespace() {
        i -= 1;
    }

    i
}

fn next_word_boundary(value: &str, cursor: usize) -> usize {
    let chars: Vec<char> = value.chars().collect();
    let mut i = cursor.min(chars.len());

    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    while i < chars.len() && !chars[i].is_whitespace() {
        i += 1;
    }

    i
}

#[derive(Clone, Debug, Default)]
pub struct FocusState {
    focused: Option<FocusId>,
    focused_path: Option<FocusPath>,
    last_child_by_parent: HashMap<FocusPath, FocusPath>,
    esc_armed: bool,
}

impl FocusState {
    pub fn focused(&self) -> Option<FocusId> {
        self.focused
    }

    pub fn focused_path(&self) -> Option<&FocusPath> {
        self.focused_path.as_ref()
    }

    pub fn is_focused(&self, id: FocusId) -> bool {
        self.focused == Some(id)
    }

    pub fn set_focused(&mut self, id: FocusId) {
        self.focused = Some(id);
        self.focused_path = None;
    }

    pub fn set_focused_entry(&mut self, entry: &FocusEntry) {
        self.focused = Some(entry.id);
        self.focused_path = Some(entry.path.clone());
    }

    pub fn clear_focus(&mut self) {
        self.focused = None;
        self.focused_path = None;
        self.last_child_by_parent.clear();
        self.esc_armed = false;
    }

    pub fn ensure_valid(&mut self, entries: &[FocusEntry]) {
        if entries.is_empty() {
            self.focused = None;
            self.focused_path = None;
            return;
        }

        if let Some(path) = &self.focused_path
            && let Some(entry) = entries.iter().find(|entry| &entry.path == path)
        {
            self.focused = Some(entry.id);
            return;
        }

        if let Some(id) = self.focused
            && let Some(entry) = entries.iter().find(|entry| entry.id == id)
        {
            self.focused = Some(entry.id);
            self.focused_path = Some(entry.path.clone());
            return;
        }

        self.set_focused_entry(&entries[0]);
    }

    pub fn focus_next(&mut self, entries: &[FocusEntry]) {
        if entries.is_empty() {
            self.clear_focus();
            return;
        }

        let idx = self.current_index(entries).unwrap_or(0).saturating_add(1) % entries.len();
        self.set_focused_entry(&entries[idx]);
    }

    pub fn focus_prev(&mut self, entries: &[FocusEntry]) {
        if entries.is_empty() {
            self.clear_focus();
            return;
        }
        let idx = match self.current_index(entries) {
            Some(0) | None => entries.len() - 1,
            Some(i) => i - 1,
        };
        self.set_focused_entry(&entries[idx]);
    }

    pub fn focus_next_sibling(&mut self, entries: &[FocusEntry]) -> bool {
        self.focus_sibling(entries, true)
    }

    pub fn focus_prev_sibling(&mut self, entries: &[FocusEntry]) -> bool {
        self.focus_sibling(entries, false)
    }

    pub fn focus_next_peer_branch(&mut self, entries: &[FocusEntry]) -> bool {
        self.focus_peer_branch(entries, true)
    }

    pub fn focus_prev_peer_branch(&mut self, entries: &[FocusEntry]) -> bool {
        self.focus_peer_branch(entries, false)
    }

    pub fn focus_parent(&mut self, entries: &[FocusEntry]) -> bool {
        let Some(path) = self.focused_path.clone() else {
            return false;
        };
        for depth in (1..path.0.len()).rev() {
            let ancestor = FocusPath(path.0[..depth].to_vec());
            if let Some(entry) = entries.iter().find(|entry| entry.path == ancestor) {
                if matches!(entry.kind, FocusKind::ScrollRegion) {
                    self.last_child_by_parent.insert(ancestor, path.clone());
                }
                self.set_focused_entry(entry);
                return true;
            }
        }
        false
    }

    pub fn focused_entry<'a>(&self, entries: &'a [FocusEntry]) -> Option<&'a FocusEntry> {
        self.current_index(entries).map(|idx| &entries[idx])
    }

    pub fn focus_first_child(&mut self, entries: &[FocusEntry]) -> bool {
        let Some(current_idx) = self.current_index(entries) else {
            return false;
        };
        let current = &entries[current_idx];
        if matches!(current.kind, FocusKind::ScrollRegion) {
            if let Some(saved_child) = self.last_child_by_parent.get(&current.path)
                && let Some(entry) = entries.iter().find(|entry| entry.path == *saved_child)
            {
                self.set_focused_entry(entry);
                return true;
            }
        }
        let mut candidates = entries
            .iter()
            .filter(|entry| {
                entry.path.0.len() > current.path.0.len()
                    && entry.path.0.get(..current.path.0.len()) == Some(current.path.0.as_slice())
            })
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return false;
        }
        candidates.sort_by(|a, b| {
            a.path
                .0
                .len()
                .cmp(&b.path.0.len())
                .then_with(|| a.path.0.cmp(&b.path.0))
        });
        self.set_focused_entry(candidates[0]);
        true
    }

    fn current_index(&self, entries: &[FocusEntry]) -> Option<usize> {
        if let Some(path) = &self.focused_path
            && let Some(idx) = entries.iter().position(|entry| &entry.path == path)
        {
            return Some(idx);
        }
        self.focused
            .and_then(|id| entries.iter().position(|entry| entry.id == id))
    }

    fn focus_sibling(&mut self, entries: &[FocusEntry], next: bool) -> bool {
        let Some(current_idx) = self.current_index(entries) else {
            return false;
        };
        let current = &entries[current_idx];
        let mut siblings = entries
            .iter()
            .filter(|entry| {
                entry.path.0.len() == current.path.0.len()
                    && entry.path.0.get(..entry.path.0.len().saturating_sub(1))
                        == current.path.0.get(..current.path.0.len().saturating_sub(1))
            })
            .collect::<Vec<_>>();
        if siblings.len() <= 1 {
            return false;
        }
        siblings.sort_by_key(|entry| &entry.path.0);
        let Some(pos) = siblings.iter().position(|entry| entry.id == current.id) else {
            return false;
        };
        let target = if next {
            siblings[(pos + 1) % siblings.len()]
        } else {
            siblings[(pos + siblings.len() - 1) % siblings.len()]
        };
        self.set_focused_entry(target);
        true
    }

    fn focus_peer_branch(&mut self, entries: &[FocusEntry], next: bool) -> bool {
        let Some(current_idx) = self.current_index(entries) else {
            return false;
        };
        let current = &entries[current_idx];
        let path = &current.path.0;

        for level in (0..path.len()).rev() {
            let parent = &path[..level];
            let current_slot = path[level];

            let mut sibling_slots = entries
                .iter()
                .filter_map(|entry| {
                    if entry.path.0.len() > level && entry.path.0.get(..level) == Some(parent) {
                        Some(entry.path.0[level])
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            sibling_slots.sort_unstable();
            sibling_slots.dedup();

            let target_slot = if next {
                sibling_slots.into_iter().find(|slot| *slot > current_slot)
            } else {
                sibling_slots
                    .into_iter()
                    .rev()
                    .find(|slot| *slot < current_slot)
            };

            let Some(target_slot) = target_slot else {
                continue;
            };

            let mut branch_entries = entries
                .iter()
                .filter(|entry| {
                    entry.path.0.len() > level
                        && entry.path.0.get(..level) == Some(parent)
                        && entry.path.0[level] == target_slot
                })
                .collect::<Vec<_>>();
            if branch_entries.is_empty() {
                continue;
            }

            branch_entries.sort_by(|a, b| {
                a.path
                    .0
                    .len()
                    .cmp(&b.path.0.len())
                    .then_with(|| a.path.0.cmp(&b.path.0))
            });
            self.set_focused_entry(branch_entries[0]);
            return true;
        }

        false
    }

    pub fn handle_navigation(
        &mut self,
        event: UiInputEvent,
        entries: &[FocusEntry],
    ) -> FocusNavOutcome {
        let UiInputEvent::Key(key) = event else {
            self.esc_armed = false;
            return FocusNavOutcome::Ignored;
        };

        let focused_kind = self.focused_entry(entries).map(|entry| entry.kind);
        let out = match key {
            UiKeyInput::Esc => {
                if self.focus_parent(entries) {
                    self.esc_armed = false;
                    FocusNavOutcome::Handled
                } else if self.esc_armed {
                    self.esc_armed = false;
                    FocusNavOutcome::RequestQuit
                } else {
                    self.esc_armed = true;
                    FocusNavOutcome::Handled
                }
            }
            UiKeyInput::Tab => {
                self.focus_next(entries);
                FocusNavOutcome::Handled
            }
            UiKeyInput::BackTab => {
                self.focus_prev(entries);
                FocusNavOutcome::Handled
            }
            UiKeyInput::Enter if focused_kind != Some(FocusKind::TextInput) => {
                if self.focus_first_child(entries) {
                    FocusNavOutcome::Handled
                } else {
                    FocusNavOutcome::Ignored
                }
            }
            UiKeyInput::Left | UiKeyInput::Up if focused_kind != Some(FocusKind::TextInput) => {
                if self.focus_prev_sibling(entries) || self.focus_prev_peer_branch(entries) {
                    FocusNavOutcome::Handled
                } else {
                    FocusNavOutcome::Ignored
                }
            }
            UiKeyInput::Right | UiKeyInput::Down if focused_kind != Some(FocusKind::TextInput) => {
                if self.focus_next_sibling(entries) || self.focus_next_peer_branch(entries) {
                    FocusNavOutcome::Handled
                } else {
                    FocusNavOutcome::Ignored
                }
            }
            _ => FocusNavOutcome::Ignored,
        };

        if key != UiKeyInput::Esc {
            self.esc_armed = false;
        }
        out
    }
}

#[derive(Clone, Debug)]
pub struct FocusListState {
    item_heights: Vec<u16>,
    viewport_lines: u16,
    gap_lines: u16,
    focused_index: u16,
    scroll_offset: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct FocusListBinding {
    first_focus_id: u64,
}

impl FocusListBinding {
    pub fn new(first_focus_id: u64) -> Self {
        Self { first_focus_id }
    }

    pub fn focus_id(&self, index: u16) -> FocusId {
        FocusId(self.first_focus_id + index as u64)
    }

    pub fn focused_index(&self, focus: &FocusState, item_count: u16) -> Option<u16> {
        let id = focus.focused()?.0;
        let end = self.first_focus_id + item_count as u64;
        if (self.first_focus_id..end).contains(&id) {
            Some((id - self.first_focus_id) as u16)
        } else {
            None
        }
    }

    pub fn sync_list_from_focus(&self, focus: &FocusState, list: &mut FocusListState) {
        if let Some(index) = self.focused_index(focus, list.item_count()) {
            list.set_focused_index(index);
        }
    }

    pub fn handle_input(
        &self,
        focus: &mut FocusState,
        list: &mut FocusListState,
        event: UiInputEvent,
    ) -> bool {
        let Some(index) = self.focused_index(focus, list.item_count()) else {
            return false;
        };
        list.set_focused_index(index);

        let handled = match event {
            UiInputEvent::Key(UiKeyInput::Up) => {
                list.move_focus_by(-1);
                true
            }
            UiInputEvent::Key(UiKeyInput::Down) => {
                list.move_focus_by(1);
                true
            }
            UiInputEvent::Key(UiKeyInput::PageUp) => {
                list.move_focus_by(-(list.viewport_lines() as i16));
                true
            }
            UiInputEvent::Key(UiKeyInput::PageDown) => {
                list.move_focus_by(list.viewport_lines() as i16);
                true
            }
            UiInputEvent::Key(UiKeyInput::Home) => {
                list.set_focused_index(0);
                true
            }
            UiInputEvent::Key(UiKeyInput::End) => {
                list.set_focused_index(list.item_count().saturating_sub(1));
                true
            }
            UiInputEvent::Key(UiKeyInput::Tab | UiKeyInput::BackTab) => {
                list.ensure_focused_visible();
                true
            }
            UiInputEvent::ScrollLines(lines) if lines < 0 => {
                list.move_focus_by(-(lines.unsigned_abs() as i16));
                true
            }
            UiInputEvent::ScrollLines(lines) if lines > 0 => {
                list.move_focus_by(lines);
                true
            }
            _ => false,
        };

        if handled {
            focus.set_focused(self.focus_id(list.focused_index()));
        }
        handled
    }
}

impl FocusListState {
    pub fn new(item_heights: Vec<u16>, viewport_lines: u16, gap_lines: u16) -> Self {
        Self {
            item_heights,
            viewport_lines: viewport_lines.max(1),
            gap_lines,
            focused_index: 0,
            scroll_offset: 0,
        }
    }

    pub fn focused_index(&self) -> u16 {
        self.focused_index
    }

    pub fn scroll_offset(&self) -> u16 {
        self.scroll_offset
    }

    pub fn viewport_lines(&self) -> u16 {
        self.viewport_lines
    }

    pub fn set_viewport_lines(&mut self, viewport_lines: u16) {
        self.viewport_lines = viewport_lines.max(1);
        self.ensure_focused_visible();
    }

    pub fn item_count(&self) -> u16 {
        self.item_heights.len() as u16
    }

    pub fn set_item_heights(&mut self, item_heights: Vec<u16>) {
        self.item_heights = item_heights;
        if self.item_heights.is_empty() {
            self.focused_index = 0;
            self.scroll_offset = 0;
            return;
        }
        self.focused_index = self.focused_index.min(self.item_count().saturating_sub(1));
        self.ensure_focused_visible();
    }

    pub fn max_scroll_offset(&self) -> u16 {
        self.content_lines().saturating_sub(self.viewport_lines)
    }

    pub fn content_lines(&self) -> u16 {
        let mut lines = 0u16;
        for (i, height) in self.item_heights.iter().copied().enumerate() {
            lines = lines.saturating_add(height);
            if i + 1 < self.item_heights.len() {
                lines = lines.saturating_add(self.gap_lines);
            }
        }
        lines
    }

    pub fn item_height(&self, index: u16) -> u16 {
        self.item_heights
            .get(index as usize)
            .copied()
            .unwrap_or(1)
            .max(1)
    }

    pub fn item_top_line(&self, index: u16) -> u16 {
        let mut top = 0u16;
        for i in 0..index.min(self.item_count()) {
            top = top
                .saturating_add(self.item_height(i))
                .saturating_add(self.gap_lines);
        }
        top
    }

    pub fn set_focused_index(&mut self, index: u16) {
        self.focused_index = index.min(self.item_count().saturating_sub(1));
        self.ensure_focused_visible();
    }

    pub fn move_focus_by(&mut self, delta: i16) {
        let current = self.focused_index as i16;
        let next = (current + delta).clamp(0, self.item_count().saturating_sub(1) as i16) as u16;
        self.set_focused_index(next);
    }

    pub fn ensure_focused_visible(&mut self) {
        let top = self.item_top_line(self.focused_index);
        let height = self.item_height(self.focused_index);
        let bottom = top.saturating_add(height);

        if top < self.scroll_offset {
            self.scroll_offset = top;
        } else {
            let viewport_end = self.scroll_offset.saturating_add(self.viewport_lines);
            if bottom > viewport_end {
                if height >= self.viewport_lines {
                    self.scroll_offset = top;
                } else {
                    self.scroll_offset = bottom.saturating_sub(self.viewport_lines);
                }
            }
        }

        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 80.0,
            height: 24.0,
        }
    }
}

#[cfg(feature = "backend-cpui")]
pub fn run_cpui<A: UiApp + 'static>(app: A) {
    crate::backends::run_cpui(app, WindowSize::default())
}

#[cfg(feature = "backend-cpui")]
pub fn run_cpui_with_size<A: UiApp + 'static>(app: A, size: WindowSize) {
    crate::backends::run_cpui(app, size)
}

pub fn run_gpui<A: UiApp + 'static>(app: A) {
    crate::backends::run_gpui(app, WindowSize::default())
}

pub fn run_gpui_with_size<A: UiApp + 'static>(app: A, size: WindowSize) {
    crate::backends::run_gpui(app, size)
}
