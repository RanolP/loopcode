use crate::style::{BoxStyle, TextStyle};

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
