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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconName {
    Search,
    Send,
    Robot,
    Info,
    Warning,
    Error,
    Check,
    ChevronRight,
    ChevronDown,
}

#[derive(Clone, Debug)]
pub struct Icon {
    pub name: IconName,
    pub color: Option<Rgb>,
    pub asset_path: Option<String>,
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
    pub gutter_highlighted: bool,
    pub visible_offset_lines: u16,
}

impl TextInput {
    pub fn to_wrapped_rich_text(&self, total_width: usize) -> RichText {
        let line_number_style = TextStyle::new().color(Rgb(0x6e7681));
        let pipe_style = if self.gutter_highlighted {
            TextStyle::new().color(Rgb(0x2f81f7))
        } else {
            TextStyle::new().color(Rgb(0x6e7681))
        };
        let mut runs = Vec::new();
        let (gutter_digits, rows) = self.wrapped_rows(total_width.saturating_sub(3));

        for row in rows {
            if !runs.is_empty() {
                runs.push(TextRun {
                    text: "\n".to_string(),
                    style: TextStyle::default(),
                });
            }
            let show_idx = row.visible_label_row(self.visible_offset_lines as usize);
            let line_label = if row.row_in_line == show_idx {
                format!("{:>width$}", row.line_number, width = gutter_digits)
            } else {
                format!("{:>width$}", "", width = gutter_digits)
            };
            runs.push(TextRun {
                text: line_label,
                style: line_number_style.clone(),
            });
            runs.push(TextRun {
                text: if self.gutter_highlighted && row.is_cursor_line && row.row_in_line == show_idx {
                    " > ".to_string()
                } else {
                    " | ".to_string()
                },
                style: pipe_style.clone(),
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
            let show_idx = row.visible_label_row(self.visible_offset_lines as usize);
            let text = if row.row_in_line == show_idx {
                format!("{:>width$}", row.line_number, width = gutter_digits)
            } else {
                format!("{:>width$}", "", width = gutter_digits)
            };
            runs.push(TextRun {
                text,
                style: line_number_style.clone(),
            });
        }

        RichText { runs }
    }

    pub fn to_wrapped_gutter_with_pipe_rich_text(&self, total_width: usize) -> RichText {
        let line_number_style = TextStyle::new().color(Rgb(0x6e7681));
        let pipe_style = if self.gutter_highlighted {
            TextStyle::new().color(Rgb(0x2f81f7))
        } else {
            TextStyle::new().color(Rgb(0x6e7681))
        };
        let mut runs = Vec::new();
        let (gutter_digits, rows) = self.wrapped_rows(total_width.saturating_sub(3));

        for row in rows {
            if !runs.is_empty() {
                runs.push(TextRun {
                    text: "\n".to_string(),
                    style: TextStyle::default(),
                });
            }
            let show_idx = row.visible_label_row(self.visible_offset_lines as usize);
            let number = if row.row_in_line == show_idx {
                format!("{:>width$}", row.line_number, width = gutter_digits)
            } else {
                format!("{:>width$}", "", width = gutter_digits)
            };
            runs.push(TextRun {
                text: number,
                style: line_number_style.clone(),
            });
            runs.push(TextRun {
                text: if self.gutter_highlighted && row.is_cursor_line && row.row_in_line == show_idx {
                    " > ".to_string()
                } else {
                    " | ".to_string()
                },
                style: pipe_style.clone(),
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
        let placeholder_style = TextStyle::new().italic().color(Rgb(0x6e7681));
        let lines: Vec<&str> = self.value.split('\n').collect();
        let line_count = lines.len().max(1);
        let gutter_digits = line_count.to_string().len();
        let content_width = total_width.saturating_sub(gutter_digits).max(1);
        let (cursor_line, cursor_col) = if self.gutter_highlighted {
            cursor_line_col(&self.value, self.cursor)
        } else {
            (0, 0)
        };

        let mut out = Vec::new();
        let mut global_row_index = 0usize;
        for (line_idx, line) in lines.iter().enumerate() {
            let mut styled_chars: Vec<(char, TextStyle)> = Vec::new();
            let chars: Vec<char> = line.chars().collect();
            if self.focused && cursor_line == line_idx {
                let col = cursor_col.min(chars.len());
                if chars.is_empty() {
                    styled_chars.push((' ', TextStyle::new().cursor_anchor(false)));
                } else if col >= chars.len() {
                    for (idx, ch) in chars.iter().copied().enumerate() {
                        let style = if idx + 1 == chars.len() {
                            TextStyle::new().cursor_anchor(true)
                        } else {
                            TextStyle::default()
                        };
                        styled_chars.push((ch, style));
                    }
                } else {
                    for (idx, ch) in chars.iter().copied().enumerate() {
                        let style = if idx == col {
                            TextStyle::new().cursor_anchor(false)
                        } else {
                            TextStyle::default()
                        };
                        styled_chars.push((ch, style));
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
            let wrapped_len = wrapped.len().max(1);

            for (row_idx, row) in wrapped.into_iter().enumerate() {
                out.push(WrappedRow {
                    line_number: line_idx + 1,
                    is_cursor_line: self.gutter_highlighted && cursor_line == line_idx,
                    row_in_line: row_idx,
                    line_rows: wrapped_len,
                    global_row: global_row_index,
                    content: row,
                });
                global_row_index += 1;
            }
        }
        (gutter_digits, out)
    }
}

#[derive(Clone)]
struct WrappedRow {
    line_number: usize,
    is_cursor_line: bool,
    row_in_line: usize,
    line_rows: usize,
    global_row: usize,
    content: Vec<(char, TextStyle)>,
}

impl WrappedRow {
    fn visible_label_row(&self, offset: usize) -> usize {
        let line_start = self.global_row.saturating_sub(self.row_in_line);
        let line_end = line_start.saturating_add(self.line_rows);
        if (line_start..line_end).contains(&offset) {
            offset - line_start
        } else {
            0
        }
    }
}

fn cursor_line_col(value: &str, cursor: usize) -> (usize, usize) {
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
    Icon(Icon),
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
                        kind: FocusKind::ScrollRegion,
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
            Node::RichText(_) | Node::Icon(_) | Node::Empty => {}
        }
    }
}
