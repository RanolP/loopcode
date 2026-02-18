use std::collections::HashMap;
use std::io;

use taffy::prelude::*;

use crate::{
    color::Rgba,
    geometry::Pixels,
    text::{StyledText, styled_text},
};

#[derive(Clone, Copy, Debug, Default)]
enum LayoutDisplay {
    #[default]
    Flex,
    Grid,
}

#[derive(Clone, Debug)]
pub enum AnyElement {
    Div(Div),
    Text(String),
    InlineText(StyledText),
    Empty,
}

pub trait IntoElement {
    fn into_any_element(self) -> AnyElement;
}

impl IntoElement for AnyElement {
    fn into_any_element(self) -> AnyElement {
        self
    }
}

impl IntoElement for Div {
    fn into_any_element(self) -> AnyElement {
        AnyElement::Div(self)
    }
}

impl IntoElement for String {
    fn into_any_element(self) -> AnyElement {
        AnyElement::Text(self)
    }
}

impl IntoElement for &str {
    fn into_any_element(self) -> AnyElement {
        AnyElement::Text(self.to_string())
    }
}

impl IntoElement for StyledText {
    fn into_any_element(self) -> AnyElement {
        AnyElement::InlineText(self)
    }
}

#[derive(Clone, Debug)]
pub struct Style {
    pub text_color: Option<Rgba>,
    pub bg: Option<Rgba>,
    display: LayoutDisplay,
    flex_direction: FlexDirection,
    justify_content: Option<JustifyContent>,
    align_items: Option<AlignItems>,
    gap: f32,
    width: Option<Pixels>,
    height: Option<Pixels>,
    grid_columns: Option<u16>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            text_color: None,
            bg: None,
            display: LayoutDisplay::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: None,
            align_items: None,
            gap: 0.0,
            width: None,
            height: None,
            grid_columns: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Div {
    style: Style,
    children: Vec<AnyElement>,
}

pub fn div() -> Div {
    Div::default()
}

impl Div {
    pub fn flex(mut self) -> Self {
        self.style.display = LayoutDisplay::Flex;
        self
    }

    pub fn grid(mut self) -> Self {
        self.style.display = LayoutDisplay::Grid;
        self
    }

    pub fn grid_cols(mut self, columns: u16) -> Self {
        self.style.grid_columns = Some(columns.max(1));
        self
    }

    pub fn flex_col(mut self) -> Self {
        self.style.flex_direction = FlexDirection::Column;
        self
    }

    pub fn gap_2(mut self) -> Self {
        self.style.gap = 1.0;
        self
    }

    pub fn gap_3(mut self) -> Self {
        self.style.gap = 2.0;
        self
    }

    pub fn justify_center(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::Center);
        self
    }

    pub fn items_center(mut self) -> Self {
        self.style.align_items = Some(AlignItems::Center);
        self
    }

    pub fn shadow_lg(self) -> Self {
        self
    }

    pub fn border_1(self) -> Self {
        self
    }

    pub fn border_dashed(self) -> Self {
        self
    }

    pub fn rounded_md(self) -> Self {
        self
    }

    pub fn text_xl(self) -> Self {
        self
    }

    pub fn size_8(mut self) -> Self {
        self.style.width = Some(Pixels(8.0));
        self.style.height = Some(Pixels(8.0));
        self
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.style.width = Some(size);
        self.style.height = Some(size);
        self
    }

    pub fn h(mut self, height: Pixels) -> Self {
        self.style.height = Some(height);
        self
    }

    pub fn bg(mut self, color: Rgba) -> Self {
        self.style.bg = Some(color);
        self
    }

    pub fn text_color(mut self, color: Rgba) -> Self {
        self.style.text_color = Some(color);
        self
    }

    pub fn border_color(self, _color: Rgba) -> Self {
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

struct TextLeaf {
    node: NodeId,
    inline: StyledText,
    color: Option<Rgba>,
}

fn taffy_style_from(div: &Div) -> taffy::style::Style {
    let mut style = taffy::style::Style::default();
    style.flex_grow = 0.0;
    style.flex_shrink = 0.0;

    style.display = match div.style.display {
        LayoutDisplay::Flex => Display::Flex,
        LayoutDisplay::Grid => Display::Grid,
    };
    style.flex_direction = div.style.flex_direction;
    style.justify_content = div.style.justify_content;
    style.align_items = div.style.align_items;
    style.gap = Size {
        width: LengthPercentage::from_length(div.style.gap),
        height: LengthPercentage::from_length(div.style.gap),
    };

    style.size = Size {
        width: div
            .style
            .width
            .map(|w| Dimension::Length(w.0))
            .unwrap_or(Dimension::Auto),
        height: div
            .style
            .height
            .map(|h| Dimension::Length(h.0))
            .unwrap_or(Dimension::Auto),
    };

    if let Some(columns) = div.style.grid_columns {
        style.grid_template_columns = (0..columns).map(|_| fr(1.0)).collect();
    }

    style
}

fn build_layout_tree(
    taffy: &mut TaffyTree<()>,
    element: &AnyElement,
    inherited_color: Option<Rgba>,
    leaves: &mut Vec<TextLeaf>,
    parents: &mut HashMap<NodeId, NodeId>,
) -> io::Result<NodeId> {
    match element {
        AnyElement::Empty => taffy
            .new_leaf(taffy::style::Style::default())
            .map_err(io::Error::other),
        AnyElement::Text(text) => {
            let inline = styled_text(text.clone());
            let style = taffy::style::Style {
                flex_grow: 0.0,
                flex_shrink: 0.0,
                size: Size {
                    width: Dimension::Length(inline.width_chars() as f32),
                    height: Dimension::Length(inline.height_lines() as f32),
                },
                ..Default::default()
            };
            let node = taffy.new_leaf(style).map_err(io::Error::other)?;
            leaves.push(TextLeaf {
                node,
                inline,
                color: inherited_color,
            });
            Ok(node)
        }
        AnyElement::InlineText(inline) => {
            let style = taffy::style::Style {
                flex_grow: 0.0,
                flex_shrink: 0.0,
                size: Size {
                    width: Dimension::Length(inline.width_chars() as f32),
                    height: Dimension::Length(inline.height_lines() as f32),
                },
                ..Default::default()
            };
            let node = taffy.new_leaf(style).map_err(io::Error::other)?;
            leaves.push(TextLeaf {
                node,
                inline: inline.clone(),
                color: inherited_color,
            });
            Ok(node)
        }
        AnyElement::Div(div) => {
            let child_color = div.style.text_color.or(inherited_color);
            let mut child_nodes = Vec::with_capacity(div.children.len());
            for child in &div.children {
                child_nodes.push(build_layout_tree(
                    taffy,
                    child,
                    child_color,
                    leaves,
                    parents,
                )?);
            }
            let node = taffy
                .new_with_children(taffy_style_from(div), &child_nodes)
                .map_err(io::Error::other)?;
            for child in child_nodes {
                parents.insert(child, node);
            }
            Ok(node)
        }
    }
}

pub(crate) fn render_element(
    out: &mut impl io::Write,
    element: &AnyElement,
    terminal_width: u16,
    terminal_height: u16,
) -> io::Result<()> {
    let mut taffy = TaffyTree::new();
    let mut leaves = Vec::new();
    let mut parents = HashMap::new();

    let root = build_layout_tree(&mut taffy, element, None, &mut leaves, &mut parents)?;

    taffy
        .compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(terminal_width as f32),
                height: AvailableSpace::Definite(terminal_height as f32),
            },
        )
        .map_err(io::Error::other)?;

    let mut absolute_cache: HashMap<NodeId, (f32, f32)> = HashMap::new();

    for leaf in leaves {
        let (abs_x, abs_y) = absolute_location(leaf.node, &taffy, &parents, &mut absolute_cache)?;
        let x = abs_x.max(0.0) as u16;
        let y = abs_y.max(0.0) as u16;
        leaf.inline.render_at(out, x, y, leaf.color)?;
    }

    Ok(())
}

fn absolute_location(
    node: NodeId,
    taffy: &TaffyTree<()>,
    parents: &HashMap<NodeId, NodeId>,
    cache: &mut HashMap<NodeId, (f32, f32)>,
) -> io::Result<(f32, f32)> {
    if let Some(loc) = cache.get(&node).copied() {
        return Ok(loc);
    }

    let layout = taffy.layout(node).map_err(io::Error::other)?;
    let own = (layout.location.x, layout.location.y);

    let abs = if let Some(parent) = parents.get(&node).copied() {
        let parent_abs = absolute_location(parent, taffy, parents, cache)?;
        (parent_abs.0 + own.0, parent_abs.1 + own.1)
    } else {
        own
    };

    cache.insert(node, abs);
    Ok(abs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_leaf_positions(
        element: &AnyElement,
        width: f32,
        height: f32,
    ) -> io::Result<HashMap<String, (u16, u16)>> {
        let mut taffy = TaffyTree::new();
        let mut leaves = Vec::new();
        let mut parents = HashMap::new();

        let root = build_layout_tree(&mut taffy, element, None, &mut leaves, &mut parents)?;
        taffy
            .compute_layout(
                root,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::Definite(height),
                },
            )
            .map_err(io::Error::other)?;

        let mut cache = HashMap::new();
        let mut out = HashMap::new();
        for leaf in leaves {
            let (x, y) = absolute_location(leaf.node, &taffy, &parents, &mut cache)?;
            let text = leaf
                .inline
                .runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>();
            out.insert(text, (x as u16, y as u16));
        }
        Ok(out)
    }

    #[test]
    fn nested_children_use_absolute_positions() -> io::Result<()> {
        let tree = div().flex_col().child("header").child(
            div()
                .flex_col()
                .child("inner-1")
                .child(div().flex_col().child("deep-1").child("deep-2")),
        );

        let pos = text_leaf_positions(&tree.into_any_element(), 80.0, 24.0)?;
        let header_y = pos["header"].1;
        let inner_1_y = pos["inner-1"].1;
        let deep_1_y = pos["deep-1"].1;
        let deep_2_y = pos["deep-2"].1;

        assert!(inner_1_y > header_y);
        assert!(deep_1_y > inner_1_y);
        assert!(deep_2_y > deep_1_y);
        Ok(())
    }

    #[test]
    fn multiline_text_reserves_height_for_following_rows() -> io::Result<()> {
        let tree = div()
            .flex_col()
            .child("row-a\nrow-a-detail")
            .child("row-b-single");

        let pos = text_leaf_positions(&tree.into_any_element(), 80.0, 24.0)?;
        let first_y = pos["row-a\nrow-a-detail"].1;
        let second_y = pos["row-b-single"].1;

        assert!(second_y >= first_y + 2);
        Ok(())
    }
}
