use crate::{
    runtime::{FocusEntry, FocusKind, FocusPath},
    style::{BoxStyle, Rgb, TextStyle},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FocusId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Row,
    Column,
}

#[derive(Clone, Debug)]
pub struct Stack {
    pub axis: Axis,
    pub gap: u8,
    pub justify_center: bool,
    pub items_center: bool,
    pub children: Vec<Node>,
}

impl Stack {
    pub fn new(axis: Axis) -> Self {
        Self {
            axis,
            gap: 0,
            justify_center: false,
            items_center: false,
            children: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Container {
    pub style: BoxStyle,
    pub focus_id: Option<FocusId>,
    pub child: Box<Node>,
}

#[derive(Clone, Debug)]
pub struct ScrollView {
    pub focus_id: Option<FocusId>,
    pub viewport_lines: Option<u16>,
    pub offset_lines: u16,
    pub child: Box<Node>,
}

#[derive(Clone, Debug)]
pub struct RichText {
    pub runs: Vec<TextRun>,
}

impl RichText {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            runs: vec![TextRun {
                text: text.into(),
                style: TextStyle::default(),
            }],
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextRun {
    pub text: String,
    pub style: TextStyle,
}

#[derive(Clone, Debug)]
pub struct TextInput {
    pub focus_id: Option<FocusId>,
    pub value: String,
    pub placeholder: Option<String>,
    pub cursor: usize,
    pub focused: bool,
}

impl TextInput {
    pub fn to_wrapped_rich_text(&self, total_width: usize) -> RichText {
        let line_number_style = TextStyle::new().color(Rgb(0x6e7681));
        let mut runs = Vec::new();
        let (gutter_digits, rows) = self.wrapped_rows(total_width.saturating_sub(3));

        for row in rows {
            if !runs.is_empty() {
                runs.push(TextRun {
                    text: "\n".to_string(),
                    style: TextStyle::default(),
                });
            }
            let prefix = match row.line_number {
                Some(line) => format!("{:>width$} | ", line, width = gutter_digits),
                None => format!("{:>width$} | ", "", width = gutter_digits),
            };
            runs.push(TextRun {
                text: prefix,
                style: line_number_style.clone(),
            });
            for (ch, style) in row.content {
                runs.push(TextRun {
                    text: ch.to_string(),
                    style,
                });
            }
        }

        RichText { runs }
    }

    pub fn to_wrapped_gutter_rich_text(&self, total_width: usize) -> RichText {
        let line_number_style = TextStyle::new().color(Rgb(0x6e7681));
        let mut runs = Vec::new();
        let (gutter_digits, rows) = self.wrapped_rows(total_width.saturating_sub(1));

        for row in rows {
            if !runs.is_empty() {
                runs.push(TextRun {
                    text: "\n".to_string(),
                    style: TextStyle::default(),
                });
            }
            let text = match row.line_number {
                Some(line) => format!("{:>width$}", line, width = gutter_digits),
                None => format!("{:>width$}", "", width = gutter_digits),
            };
            runs.push(TextRun {
                text,
                style: line_number_style.clone(),
            });
        }

        RichText { runs }
    }

    pub fn to_wrapped_content_rich_text(&self, total_width: usize) -> RichText {
        let mut runs = Vec::new();
        let (_, rows) = self.wrapped_rows(total_width.saturating_sub(1));
        for row in rows {
            if !runs.is_empty() {
                runs.push(TextRun {
                    text: "\n".to_string(),
                    style: TextStyle::default(),
                });
            }
            for (ch, style) in row.content {
                runs.push(TextRun {
                    text: ch.to_string(),
                    style,
                });
            }
        }
        RichText { runs }
    }

    fn wrapped_rows(&self, total_width: usize) -> (usize, Vec<WrappedRow>) {
        let cursor_style = TextStyle::new().bg(Rgb(0x2f81f7)).color(Rgb(0x0d1117));
        let placeholder_style = TextStyle::new().italic().color(Rgb(0x6e7681));
        let lines: Vec<&str> = self.value.split('\n').collect();
        let line_count = lines.len().max(1);
        let gutter_digits = line_count.to_string().len();
        let content_width = total_width.saturating_sub(gutter_digits).max(1);
        let (cursor_line, cursor_col) = if self.focused {
            cursor_line_col(&self.value, self.cursor)
        } else {
            (0, 0)
        };

        let mut out = Vec::new();
        for (line_idx, line) in lines.iter().enumerate() {
            let mut styled_chars: Vec<(char, TextStyle)> = Vec::new();
            let chars: Vec<char> = line.chars().collect();
            if self.focused && cursor_line == line_idx {
                let col = cursor_col.min(chars.len());
                for ch in &chars[..col] {
                    styled_chars.push((*ch, TextStyle::default()));
                }
                styled_chars.push((chars.get(col).copied().unwrap_or(' '), cursor_style.clone()));
                if col < chars.len() {
                    for ch in &chars[col + 1..] {
                        styled_chars.push((*ch, TextStyle::default()));
                    }
                }
            } else if line.is_empty() && !self.focused && self.value.is_empty() {
                if let Some(placeholder) = &self.placeholder {
                    for ch in placeholder.chars() {
                        styled_chars.push((ch, placeholder_style.clone()));
                    }
                }
            } else {
                for ch in chars {
                    styled_chars.push((ch, TextStyle::default()));
                }
            }

            let wrapped = wrap_styled_chars(&styled_chars, content_width);
            if wrapped.is_empty() {
                out.push(WrappedRow {
                    line_number: Some(line_idx + 1),
                    content: Vec::new(),
                });
                continue;
            }

            for (row_idx, row) in wrapped.into_iter().enumerate() {
                out.push(WrappedRow {
                    line_number: if row_idx == 0 {
                        Some(line_idx + 1)
                    } else {
                        None
                    },
                    content: row,
                });
            }
        }
        (gutter_digits, out)
    }
}

#[derive(Clone)]
struct WrappedRow {
    line_number: Option<usize>,
    content: Vec<(char, TextStyle)>,
}

fn cursor_line_col(value: &str, cursor: usize) -> (usize, usize) {
    let mut line = 0usize;
    let mut col = 0usize;
    let mut i = 0usize;
    for ch in value.chars() {
        if i == cursor {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        i += 1;
    }
    (line, col)
}

fn wrap_styled_chars(chars: &[(char, TextStyle)], width: usize) -> Vec<Vec<(char, TextStyle)>> {
    if width == 0 {
        return vec![chars.to_vec()];
    }

    let mut rows: Vec<Vec<(char, TextStyle)>> = Vec::new();
    let mut row: Vec<(char, TextStyle)> = Vec::new();
    let mut row_width = 0usize;

    for (ch, style) in chars.iter().cloned() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if row_width > 0 && row_width.saturating_add(ch_width) > width {
            rows.push(std::mem::take(&mut row));
            row_width = 0;
        }
        row.push((ch, style));
        row_width = row_width.saturating_add(ch_width);
    }

    rows.push(row);
    rows
}

#[derive(Clone, Debug)]
pub enum Node {
    Stack(Stack),
    Container(Container),
    ScrollView(ScrollView),
    RichText(RichText),
    TextInput(TextInput),
    Empty,
}

pub trait IntoNode {
    fn into_node(self) -> Node;
}

impl IntoNode for Node {
    fn into_node(self) -> Node {
        self
    }
}

impl Node {
    pub fn collect_focus_ids(&self, out: &mut Vec<FocusId>) {
        let mut entries = Vec::new();
        self.collect_focus_entries(&mut entries);
        out.extend(entries.into_iter().map(|entry| entry.id));
    }

    pub fn collect_focus_entries(&self, out: &mut Vec<FocusEntry>) {
        let mut path = Vec::new();
        self.collect_focus_entries_inner(out, &mut path);
    }

    fn collect_focus_entries_inner(&self, out: &mut Vec<FocusEntry>, path: &mut Vec<usize>) {
        match self {
            Node::Stack(stack) => {
                for (i, child) in stack.children.iter().enumerate() {
                    path.push(i);
                    child.collect_focus_entries_inner(out, path);
                    path.pop();
                }
            }
            Node::Container(container) => {
                if let Some(id) = container.focus_id {
                    out.push(FocusEntry {
                        id,
                        path: FocusPath(path.clone()),
                        kind: FocusKind::Generic,
                    });
                }
                path.push(0);
                container.child.collect_focus_entries_inner(out, path);
                path.pop();
            }
            Node::ScrollView(scroll) => {
                if let Some(id) = scroll.focus_id {
                    out.push(FocusEntry {
                        id,
                        path: FocusPath(path.clone()),
                        kind: FocusKind::Generic,
                    });
                }
                path.push(0);
                scroll.child.collect_focus_entries_inner(out, path);
                path.pop();
            }
            Node::TextInput(input) => {
                if let Some(id) = input.focus_id {
                    out.push(FocusEntry {
                        id,
                        path: FocusPath(path.clone()),
                        kind: FocusKind::TextInput,
                    });
                }
            }
            Node::RichText(_) | Node::Empty => {}
        }
    }
}
