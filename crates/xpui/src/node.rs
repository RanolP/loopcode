use crate::style::{BoxStyle, TextStyle};

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
pub enum Node {
    Stack(Stack),
    Container(Container),
    ScrollView(ScrollView),
    RichText(RichText),
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
        match self {
            Node::Stack(stack) => {
                for child in &stack.children {
                    child.collect_focus_ids(out);
                }
            }
            Node::Container(container) => {
                if let Some(id) = container.focus_id {
                    out.push(id);
                }
                container.child.collect_focus_ids(out);
            }
            Node::ScrollView(scroll) => {
                if let Some(id) = scroll.focus_id {
                    out.push(id);
                }
                scroll.child.collect_focus_ids(out);
            }
            Node::RichText(_) | Node::Empty => {}
        }
    }
}
