use crate::{
    node::{Axis, Container, IntoNode, Node, Stack},
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
    child: Node,
}

impl ContainerWidget {
    pub fn new(child: impl IntoNode) -> Self {
        Self {
            style: BoxStyle::default(),
            child: child.into_node(),
        }
    }

    pub fn style(mut self, style: BoxStyle) -> Self {
        self.style = style;
        self
    }
}

impl IntoNode for ContainerWidget {
    fn into_node(self) -> Node {
        Node::Container(Container {
            style: self.style,
            child: Box::new(self.child),
        })
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
