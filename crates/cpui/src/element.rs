use std::collections::HashMap;
use std::io;

use taffy::prelude::*;
use taffy::{Overflow, Point};

use crate::{
    color::Rgba,
    frame::CellBuffer,
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
    ScrollView(ScrollView),
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

impl IntoElement for ScrollView {
    fn into_any_element(self) -> AnyElement {
        AnyElement::ScrollView(self)
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

#[derive(Clone, Debug)]
pub struct ScrollView {
    viewport_lines: Option<u16>,
    offset_lines: u16,
    child: Box<AnyElement>,
}

pub fn div() -> Div {
    Div::default()
}

pub fn scroll_view(child: impl IntoElement) -> ScrollView {
    ScrollView {
        viewport_lines: None,
        offset_lines: 0,
        child: Box::new(child.into_any_element()),
    }
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

impl ScrollView {
    pub fn viewport_lines(mut self, lines: u16) -> Self {
        self.viewport_lines = Some(lines.max(1));
        self
    }

    pub fn offset_lines(mut self, lines: u16) -> Self {
        self.offset_lines = lines;
        self
    }
}

struct TextLeaf {
    node: NodeId,
    inline: StyledText,
    color: Option<Rgba>,
}

struct BgLeaf {
    node: NodeId,
    bg: Rgba,
}

struct BuildState {
    leaves: Vec<TextLeaf>,
    backgrounds: Vec<BgLeaf>,
    parents: HashMap<NodeId, NodeId>,
    scroll_nodes: HashMap<NodeId, ScrollNode>,
}

#[derive(Clone, Copy, Debug)]
struct ScrollNode {
    offset_lines: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Rect {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

impl Rect {
    fn intersect(self, other: Rect) -> Option<Rect> {
        let left = self.left.max(other.left);
        let top = self.top.max(other.top);
        let right = self.right.min(other.right);
        let bottom = self.bottom.min(other.bottom);

        if left >= right || top >= bottom {
            None
        } else {
            Some(Rect {
                left,
                top,
                right,
                bottom,
            })
        }
    }
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
    wrap_width: usize,
    inherited_color: Option<Rgba>,
    state: &mut BuildState,
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
                    width: Dimension::Length(inline.wrapped_width_chars(wrap_width) as f32),
                    height: Dimension::Length(inline.wrapped_height_lines(wrap_width) as f32),
                },
                ..Default::default()
            };
            let node = taffy.new_leaf(style).map_err(io::Error::other)?;
            state.leaves.push(TextLeaf {
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
                    width: Dimension::Length(inline.wrapped_width_chars(wrap_width) as f32),
                    height: Dimension::Length(inline.wrapped_height_lines(wrap_width) as f32),
                },
                ..Default::default()
            };
            let node = taffy.new_leaf(style).map_err(io::Error::other)?;
            state.leaves.push(TextLeaf {
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
                    wrap_width,
                    child_color,
                    state,
                )?);
            }
            let node = taffy
                .new_with_children(taffy_style_from(div), &child_nodes)
                .map_err(io::Error::other)?;
            if let Some(bg) = div.style.bg {
                state.backgrounds.push(BgLeaf { node, bg });
            }
            for child in child_nodes {
                state.parents.insert(child, node);
            }
            Ok(node)
        }
        AnyElement::ScrollView(scroll) => {
            let child = build_layout_tree(
                taffy,
                &scroll.child,
                wrap_width,
                inherited_color,
                state,
            )?;

            let style = taffy::style::Style {
                flex_grow: 0.0,
                flex_shrink: 0.0,
                overflow: Point {
                    x: Overflow::Hidden,
                    y: Overflow::Hidden,
                },
                size: Size {
                    width: Dimension::Auto,
                    height: scroll
                        .viewport_lines
                        .map(|h| Dimension::Length(h as f32))
                        .unwrap_or(Dimension::Auto),
                },
                ..Default::default()
            };

            let node = taffy
                .new_with_children(style, &[child])
                .map_err(io::Error::other)?;
            state.parents.insert(child, node);
            state.scroll_nodes.insert(
                node,
                ScrollNode {
                    offset_lines: scroll.offset_lines as f32,
                },
            );
            Ok(node)
        }
    }
}

pub(crate) fn render_element(
    element: &AnyElement,
    terminal_width: u16,
    terminal_height: u16,
) -> io::Result<CellBuffer> {
    let mut taffy = TaffyTree::new();
    let mut state = BuildState {
        leaves: Vec::new(),
        backgrounds: Vec::new(),
        parents: HashMap::new(),
        scroll_nodes: HashMap::new(),
    };

    let root = build_layout_tree(
        &mut taffy,
        element,
        terminal_width as usize,
        None,
        &mut state,
    )?;
    let mut root_style = taffy.style(root).map_err(io::Error::other)?.clone();
    root_style.size = Size {
        width: Dimension::Length(terminal_width as f32),
        height: Dimension::Length(terminal_height as f32),
    };
    taffy
        .set_style(root, root_style)
        .map_err(io::Error::other)?;

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
    let screen = Rect {
        left: 0,
        top: 0,
        right: terminal_width as i32,
        bottom: terminal_height as i32,
    };

    let mut buffer = CellBuffer::new(terminal_width, terminal_height);

    for bg in state.backgrounds {
        let (abs_x, abs_y) =
            absolute_location(bg.node, &taffy, &state.parents, &mut absolute_cache)?;
        let mut y = abs_y;
        let mut clip = Some(screen);
        let mut current = bg.node;

        while let Some(parent) = state.parents.get(&current).copied() {
            if let Some(scroll) = state.scroll_nodes.get(&parent).copied() {
                y -= scroll.offset_lines;

                let (sx, sy) =
                    absolute_location(parent, &taffy, &state.parents, &mut absolute_cache)?;
                let layout = taffy.layout(parent).map_err(io::Error::other)?;
                let bounds = Rect {
                    left: sx.floor() as i32,
                    top: sy.floor() as i32,
                    right: (sx + layout.size.width).ceil() as i32,
                    bottom: (sy + layout.size.height).ceil() as i32,
                };
                clip = clip.and_then(|existing| existing.intersect(bounds));
            }
            current = parent;
        }

        if let Some(clip) = clip {
            let layout = taffy.layout(bg.node).map_err(io::Error::other)?;
            let bounds = Rect {
                left: abs_x.floor() as i32,
                top: y.floor() as i32,
                right: (abs_x + layout.size.width).ceil() as i32,
                bottom: (y + layout.size.height).ceil() as i32,
            };
            if let Some(bounds) = bounds.intersect(clip) {
                fill_rect_bg(&mut buffer, bounds, bg.bg);
            }
        }
    }

    for leaf in state.leaves {
        let (abs_x, abs_y) =
            absolute_location(leaf.node, &taffy, &state.parents, &mut absolute_cache)?;
        let mut y = abs_y;
        let mut clip = Some(screen);
        let mut current = leaf.node;

        while let Some(parent) = state.parents.get(&current).copied() {
            if let Some(scroll) = state.scroll_nodes.get(&parent).copied() {
                y -= scroll.offset_lines;

                let (sx, sy) =
                    absolute_location(parent, &taffy, &state.parents, &mut absolute_cache)?;
                let layout = taffy.layout(parent).map_err(io::Error::other)?;
                let bounds = Rect {
                    left: sx.floor() as i32,
                    top: sy.floor() as i32,
                    right: (sx + layout.size.width).ceil() as i32,
                    bottom: (sy + layout.size.height).ceil() as i32,
                };
                clip = clip.and_then(|existing| existing.intersect(bounds));
            }
            current = parent;
        }

        if let Some(clip) = clip {
            leaf.inline.render_at_clipped(
                &mut buffer,
                abs_x.floor() as i32,
                y.floor() as i32,
                leaf.color,
                clip,
            );
        }
    }

    Ok(buffer)
}

fn fill_rect_bg(
    buffer: &mut CellBuffer,
    bounds: Rect,
    bg: Rgba,
) {
    for y in bounds.top..bounds.bottom {
        for x in bounds.left..bounds.right {
            if x < 0 || y < 0 {
                continue;
            }
            let Ok(xu) = u16::try_from(x) else {
                continue;
            };
            let Ok(yu) = u16::try_from(y) else {
                continue;
            };
            if xu >= buffer.width() || yu >= buffer.height() {
                continue;
            }
            buffer.set_bg(xu, yu, bg);
        }
    }
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
        let mut state = BuildState {
            leaves: Vec::new(),
            backgrounds: Vec::new(),
            parents: HashMap::new(),
            scroll_nodes: HashMap::new(),
        };

        let root = build_layout_tree(
            &mut taffy,
            element,
            width as usize,
            None,
            &mut state,
        )?;
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
        for leaf in state.leaves {
            let (x, y) = absolute_location(leaf.node, &taffy, &state.parents, &mut cache)?;
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
