use crate::{
    node::{Axis, Container, FocusId, IntoNode, Node, ScrollView, Stack},
    style::BoxStyle,
};

pub struct StackWidget {
    inner: Stack,
}

impl StackWidget {
    pub fn row() -> Self {
        Self {
            inner: Stack::new(Axis::Row),
        }
    }

    pub fn column() -> Self {
        Self {
            inner: Stack::new(Axis::Column),
        }
    }

    pub fn gap(mut self, gap: u8) -> Self {
        self.inner.gap = gap;
        self
    }

    pub fn justify_center(mut self) -> Self {
        self.inner.justify_center = true;
        self
    }

    pub fn items_center(mut self) -> Self {
        self.inner.items_center = true;
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.inner.children.push(child.into_node());
        self
    }
}

impl IntoNode for StackWidget {
    fn into_node(self) -> Node {
        Node::Stack(self.inner)
    }
}

pub struct ContainerWidget {
    style: BoxStyle,
    focus_id: Option<FocusId>,
    child: Node,
}

impl ContainerWidget {
    pub fn new(child: impl IntoNode) -> Self {
        Self {
            style: BoxStyle::default(),
            focus_id: None,
            child: child.into_node(),
        }
    }

    pub fn style(mut self, style: BoxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn focus(mut self, focus_id: FocusId) -> Self {
        self.focus_id = Some(focus_id);
        self
    }
}

impl IntoNode for ContainerWidget {
    fn into_node(self) -> Node {
        Node::Container(Container {
            style: self.style,
            focus_id: self.focus_id,
            child: Box::new(self.child),
        })
    }
}

pub struct ScrollViewWidget {
    inner: ScrollView,
}

impl ScrollViewWidget {
    pub fn new(child: impl IntoNode) -> Self {
        Self {
            inner: ScrollView {
                focus_id: None,
                viewport_lines: None,
                offset_lines: 0,
                child: Box::new(child.into_node()),
            },
        }
    }

    pub fn viewport_lines(mut self, lines: u16) -> Self {
        self.inner.viewport_lines = Some(lines.max(1));
        self
    }

    pub fn offset_lines(mut self, lines: u16) -> Self {
        self.inner.offset_lines = lines;
        self
    }

    pub fn focus(mut self, focus_id: FocusId) -> Self {
        self.inner.focus_id = Some(focus_id);
        self
    }
}

impl IntoNode for ScrollViewWidget {
    fn into_node(self) -> Node {
        Node::ScrollView(self.inner)
    }
}

pub fn row() -> StackWidget {
    StackWidget::row()
}

pub fn column() -> StackWidget {
    StackWidget::column()
}

pub fn container(child: impl IntoNode) -> ContainerWidget {
    ContainerWidget::new(child)
}

pub fn scroll_view(child: impl IntoNode) -> ScrollViewWidget {
    ScrollViewWidget::new(child)
}
