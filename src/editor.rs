use std::f32;

use iced::{
    Alignment, Border, Color, Element, Length, Padding, Pixels, Point, Rectangle, Shadow, Size,
    Theme, Vector,
    advanced::{
        Layout, Text, Widget, layout, mouse,
        renderer::{self, Quad},
        text::{Paragraph, Wrapping, paragraph},
        widget::{Tree, tree},
    },
    alignment,
    widget::{
        self,
        text::{self, LineHeight},
        text_editor::{Binding, Content, KeyPress},
    },
};

// #[derive(Debug, Clone)]
// pub enum Message {}

pub struct Editor<'a, Message> {
    id: Option<widget::Id>,
    font: Option<iced::Font>,
    text_size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    min_height: f32,
    max_height: f32,
    padding: Padding,
    alignment: alignment::Horizontal,
    wrapping: Wrapping,
    key_binding: Option<Box<dyn Fn(KeyPress) -> Option<Binding<Message>> + 'a>>,
    placeholder: String,
}

impl<'a, Message> Editor<'a, Message> {
    pub fn new() -> Self {
        Self {
            id: None,
            font: None,
            text_size: None,
            line_height: LineHeight::default(),
            width: Length::Fill,
            height: Length::Fill,
            min_height: 0.0,
            max_height: f32::INFINITY,
            padding: Padding::new(5.0),
            alignment: alignment::Horizontal::Left,
            wrapping: Wrapping::default(),
            key_binding: None,
            placeholder: String::from("Large text string to be seen on Screen"),
        }
    }
}

impl<'a, Message, Renderer> From<Editor<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn from(Editor: Editor<'a, Message>) -> Self {
        Self::new(Editor)
    }
}

impl<'a, Message, Renderer> Widget<Message, Theme, Renderer> for Editor<'a, Message>
where
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new())
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        // let value = value.unwrap_or(&self.value);

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
        let padding = self.padding.fit(Size::ZERO, limits.max());
        let height = self.line_height.to_absolute(text_size);

        let limits = limits.width(self.width).shrink(padding);
        let text_bounds = limits.resolve(self.width, height, Size::ZERO);

        let placeholder_text = Text {
            font,
            line_height: self.line_height,
            content: self.placeholder.as_str(),
            bounds: Size::new(f32::INFINITY, text_bounds.height),
            size: text_size,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Center,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::default(),
        };

        let _ = state.placeholder.update(placeholder_text);

        let text = layout::Node::new(text_bounds).move_to(Point::new(padding.left, padding.top));

        layout::Node::with_children(text_bounds.expand(padding), vec![text])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let value = "Value String not placeholder";
        // let is_disabled = self.on_input.is_none();

        let bounds = layout.bounds();

        let mut children_layout = layout.children();
        let text_bounds = children_layout.next().unwrap().bounds();

        // let style = theme.style(&self.class, self.last_status.unwrap_or(Status::Disabled));

        // renderer.fill_quad(
        //     renderer::Quad {
        //         bounds,
        //         // border: Bord,
        //         ..renderer::Quad::default()
        //     },
        //     Color::WHITE,
        // );

        let text = ""; //value.to_string();

        let draw = |renderer: &mut Renderer, viewport| {
            let paragraph = if text.is_empty() {
                state.placeholder.raw()
            } else {
                state.value.raw()
            };

            let alignment_offset =
                alignment_offset(text_bounds.width, paragraph.min_width(), self.alignment);

            renderer.with_translation(Vector::ZERO, |_| {});

            renderer.fill_paragraph(
                paragraph,
                text_bounds.anchor(paragraph.min_bounds(), Alignment::Start, Alignment::Center)
                    + Vector::new(alignment_offset - 0.0, 0.0),
                Color::WHITE,
                viewport,
            );
        };

        draw(renderer, text_bounds);
    }
}

#[derive(Debug, Default, Clone)]
pub struct State<P: Paragraph> {
    value: paragraph::Plain<P>,
    placeholder: paragraph::Plain<P>,
    icon: paragraph::Plain<P>,
    // is_focused: Option<Focus>,
    // is_dragging: Option<Drag>,
    // is_pasting: Option<Value>,
    // preedit: Option<input_method::Preedit>,
    last_click: Option<mouse::Click>,
    // cursor: Cursor,
    // keyboard_modifiers: keyboard::Modifiers,
    // TODO: Add stateful horizontal scrolling offset
}

impl<P: Paragraph> State<P> {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new() -> Self {
        Self::default()
    }
}

fn alignment_offset(
    text_bounds_width: f32,
    text_min_width: f32,
    alignment: alignment::Horizontal,
) -> f32 {
    if text_min_width > text_bounds_width {
        0.0
    } else {
        match alignment {
            alignment::Horizontal::Left => 0.0,
            alignment::Horizontal::Center => (text_bounds_width - text_min_width) / 2.0,
            alignment::Horizontal::Right => text_bounds_width - text_min_width,
        }
    }
}
