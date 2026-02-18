use crate::{
    node::{IntoNode, Node, RichText, TextRun},
    style::TextStyle,
};

pub struct TextWidget {
    inner: RichText,
}

impl TextWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            inner: RichText::plain(text),
        }
    }

    pub fn run(mut self, text: impl Into<String>, style: TextStyle) -> Self {
        self.inner.runs.push(TextRun {
            text: text.into(),
            style,
        });
        self
    }
}

impl IntoNode for TextWidget {
    fn into_node(self) -> Node {
        Node::RichText(self.inner)
    }
}

pub fn text(text: impl Into<String>) -> TextWidget {
    TextWidget::new(text)
}
