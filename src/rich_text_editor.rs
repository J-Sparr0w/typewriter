use std::f32;

use iced::{
    Border, Color, Element, Font, Length, Padding, Pixels, Point, Rectangle, Shadow,
    Size, Theme,
    advanced::{
        Layout, Text, Widget, input_method,
        layout, mouse,
        renderer::{self, Quad},
        text::{LineHeight, Paragraph, Renderer, Shaping, Wrapping},
        widget::{Tree, operation, tree},
        Renderer as IcedRenderer,
    },
    alignment,
    widget::{self, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    TextChanged(String),
    LineSelected(usize),
    LineMoved { from: usize, to: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RichSpan {
    pub text: String,
    pub font: Font,
    pub size: f32,
    pub color: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for RichSpan {
    fn default() -> Self {
        Self {
            text: String::new(),
            font: Font::default(),
            size: 16.0,
            color: Color::BLACK,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RichLine {
    pub spans: Vec<RichSpan>,
    pub selected: bool,
}

impl RichLine {
    pub fn new() -> Self {
        Self {
            spans: vec![RichSpan::default()],
            selected: false,
        }
    }

    pub fn text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }

    pub fn set_text(&mut self, text: String) {
        if self.spans.is_empty() {
            self.spans.push(RichSpan::default());
        }
        self.spans[0].text = text;
    }
}

pub struct RichTextEditor<'a> {
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
    on_change: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_line_select: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_line_move: Option<Box<dyn Fn(usize, usize) -> Message + 'a>>,
}

impl<'a> RichTextEditor<'a> {
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
            on_change: None,
            on_line_select: None,
            on_line_move: None,
        }
    }

    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(String) -> Message + 'a,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn on_line_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize) -> Message + 'a,
    {
        self.on_line_select = Some(Box::new(callback));
        self
    }

    pub fn on_line_move<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, usize) -> Message + 'a,
    {
        self.on_line_move = Some(Box::new(callback));
        self
    }
}

impl<'a> From<RichTextEditor<'a>> for Element<'a, Message, Theme, iced::Renderer> {
    fn from(editor: RichTextEditor<'a>) -> Self {
        Self::new(editor)
    }
}

impl<'a> Widget<Message, Theme, iced::Renderer> for RichTextEditor<'a> {
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<EditorState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(EditorState::new())
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<EditorState>();
        // // let value = value.unwrap_or(&self.value);

        // let font = self.font.unwrap_or_else(|| renderer.default_font());
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
        let padding = self.padding.fit(Size::ZERO, limits.max());
        let height = self.line_height.to_absolute(text_size);

        let limits = limits.width(self.width).shrink(padding);
        let text_bounds = limits.resolve(self.width, height, Size::ZERO);

        // let placeholder_text = Text {
        //     font,
        //     line_height: self.line_height,
        //     content: self.placeholder.as_str(),
        //     bounds: Size::new(f32::INFINITY, text_bounds.height),
        //     size: text_size,
        //     align_x: text::Alignment::Default,
        //     align_y: alignment::Vertical::Center,
        //     shaping: text::Shaping::Advanced,
        //     wrapping: text::Wrapping::default(),
        // };

        let text = layout::Node::new(text_bounds).move_to(Point::new(padding.left, padding.top));

        layout::Node::with_children(text_bounds.expand(padding), vec![text])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<EditorState>();
        let bounds = layout.bounds();
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size()).0;

        // Draw background
        renderer.fill_quad(
            Quad {
                bounds,
                ..Quad::default()
            },
            Color::WHITE,
        );

        let mut y_offset = bounds.y + self.padding.top;

        for (line_index, line) in state.content.lines.iter().enumerate() {
            let line_height = self.line_height.to_absolute(Pixels(text_size)).0;

            // Draw line selection background
            if line.selected {
                let line_bounds = Rectangle {
                    x: bounds.x,
                    y: y_offset,
                    width: bounds.width,
                    height: line_height,
                };
                renderer.fill_quad(
                    Quad {
                        bounds: line_bounds,
                        ..Quad::default()
                    },
                    Color::from_rgba(0.7, 0.9, 1.0, 0.3), // Light blue selection
                );
            }

            // Draw text for each span in the line
            let mut x_offset = bounds.x + self.padding.left;
            for span in &line.spans {
                if !span.text.is_empty() {
                    // Create font based on formatting
                    let font = if span.bold {
                        // Use bold weight
                        Font {
                            weight: iced::font::Weight::Bold,
                            ..span.font
                        }
                    } else {
                        span.font
                    };

                    let text_element = Text {
                        content: span.text.clone(),
                        bounds: Size::new(f32::INFINITY, line_height),
                        size: Pixels(span.size),
                        line_height: self.line_height,
                        font,
                        align_x: text::Alignment::Default,
                        align_y: alignment::Vertical::Top,
                        shaping: Shaping::Advanced,
                        wrapping: Wrapping::None,
                    };

                    renderer.fill_text(
                        text_element,
                        Point { x: x_offset, y: y_offset },
                        span.color,
                        *viewport,
                    );

                    // Simple width calculation (in a real implementation, you'd measure the text)
                    x_offset += span.text.len() as f32 * span.size * 0.6;
                }
            }

            // Draw cursor if this is the current line
            if line_index == state.content.current_line {
                let cursor_x = bounds.x + self.padding.left +
                    state.content.cursor_position as f32 * text_size * 0.6;
                let cursor_bounds = Rectangle {
                    x: cursor_x,
                    y: y_offset,
                    width: 2.0,
                    height: line_height,
                };
                renderer.fill_quad(
                    Quad {
                        bounds: cursor_bounds,
                        ..Quad::default()
                    },
                    Color::BLACK,
                );
            }

            y_offset += line_height;
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::Text
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<EditorState>();
        let bounds = layout.bounds();
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size()).0;
        let line_height = self.line_height.to_absolute(Pixels(text_size)).0;

        match event {
            iced::Event::Keyboard(event) => {
                match event {
                    iced::keyboard::Event::KeyPressed {
                        key,
                        modifiers,
                        text,
                        ..
                    } => {
                        let mut should_redraw = true;
                        let mut message = None;

                        match key {
                            iced::keyboard::Key::Character(c) if !modifiers.control() && !modifiers.alt() => {
                                if let Some(ch) = c.chars().next() {
                                    state.content.insert_char(ch);
                                    if let Some(ref callback) = self.on_change {
                                        message = Some(callback(state.content.text()));
                                    }
                                }
                            }
                            iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) => {
                                state.content.insert_newline();
                                if let Some(ref callback) = self.on_change {
                                    message = Some(callback(state.content.text()));
                                }
                            }
                            iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace) => {
                                state.content.delete_char();
                                if let Some(ref callback) = self.on_change {
                                    message = Some(callback(state.content.text()));
                                }
                            }
                            iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft) => {
                                if modifiers.control() {
                                    // Move to previous line
                                    state.content.move_cursor_up();
                                } else {
                                    state.content.move_cursor_left();
                                }
                            }
                            iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight) => {
                                if modifiers.control() {
                                    // Move to next line
                                    state.content.move_cursor_down();
                                } else {
                                    state.content.move_cursor_right();
                                }
                            }
                            iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                                state.content.move_cursor_up();
                            }
                            iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                                state.content.move_cursor_down();
                            }
                            iced::keyboard::Key::Named(iced::keyboard::key::Named::PageUp) if modifiers.alt() => {
                                // Move current line up
                                let current_line = state.content.current_line;
                                state.content.move_line_up(current_line);
                                if let Some(ref callback) = self.on_line_move {
                                    message = Some(callback(current_line, current_line.saturating_sub(1)));
                                }
                            }
                            iced::keyboard::Key::Named(iced::keyboard::key::Named::PageDown) if modifiers.alt() => {
                                // Move current line down
                                let current_line = state.content.current_line;
                                state.content.move_line_down(current_line);
                                if let Some(ref callback) = self.on_line_move {
                                    message = Some(callback(current_line, current_line + 1));
                                }
                            }
                            iced::keyboard::Key::Character(c) if c == "b" && modifiers.control() => {
                                // Toggle bold formatting
                                state.content.toggle_bold_at_cursor();
                                if let Some(ref callback) = self.on_change {
                                    message = Some(callback(state.content.text()));
                                }
                            }
                            _ => {
                                should_redraw = false;
                            }
                        }

                        if should_redraw {
                            shell.request_redraw();
                        }
                        if let Some(msg) = message {
                            shell.publish(msg);
                        }
                        shell.capture_event();
                    }
                    _ => {}
                }
            }
            iced::Event::Mouse(event) => {
                match event {
                    iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                        if let Some(cursor_pos) = cursor.position_in(bounds) {
                            let line_index = ((cursor_pos.y - self.padding.top) / line_height) as usize;
                            if line_index < state.content.lines.len() {
                                state.content.select_line(line_index);
                                state.content.current_line = line_index;
                                state.content.cursor_position = 0; // Reset cursor to start of line

                                if let Some(ref callback) = self.on_line_select {
                                    shell.publish(callback(line_index));
                                }
                                shell.request_redraw();
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl operation::TextInput for EditorState {
    fn text(&self) -> &str {
        // For simplicity, return the text of the current line
        // In a full implementation, this would need to handle the entire document
        if let Some(line) = self.content.lines.get(self.content.current_line) {
            // This is a temporary solution - ideally we'd store the text as a String field
            // For now, we'll return an empty string to avoid the borrow checker issue
            ""
        } else {
            ""
        }
    }

    fn move_cursor_to_front(&mut self) {
        self.content.current_line = 0;
        self.content.cursor_position = 0;
    }

    fn move_cursor_to_end(&mut self) {
        self.content.current_line = self.content.lines.len().saturating_sub(1);
        self.content.cursor_position = self.content.lines[self.content.current_line].spans[0].text.len();
    }

    fn move_cursor_to(&mut self, position: usize) {
        // Simplified - would need proper text position calculation
        self.move_cursor_to_front();
        for _ in 0..position {
            self.content.move_cursor_right();
        }
    }

    fn select_all(&mut self) {
        for line in &mut self.content.lines {
            line.selected = true;
        }
    }

    fn select_range(&mut self, start: usize, end: usize) {
        // Simplified selection - would need proper implementation
        self.select_all();
    }
}

fn measure_cursor_and_scroll_offset(
    paragraph: &impl Paragraph,
    text_bounds: Rectangle,
    cursor_index: usize,
) -> (f32, f32) {
    let grapheme_position = paragraph
        .grapheme_position(0, cursor_index)
        .unwrap_or(Point::ORIGIN);

    let offset = ((grapheme_position.x + 5.0) - text_bounds.width).max(0.0);

    (grapheme_position.x, offset)
}

#[derive(Debug, Clone)]
pub enum Cursor {
    Index(usize),
    Selection(usize, usize),
}

impl Default for Cursor {
    fn default() -> Self {
        Self::Index(0)
    }
}

impl Cursor {
    pub fn inc(&mut self) {
        match self {
            Cursor::Index(index) => *index += 1,
            Cursor::Selection(_, end) => *end += 1,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct EditorState {
    content: EditorContent,
    last_span_start: usize,
    preedit: Option<input_method::Preedit>,
    last_click: Option<mouse::Click>,
    drag_start: Option<Point>,
    is_dragging: bool,
}

impl EditorState {
    /// Creates a new [`EditorState`], representing an unfocused [`RichTextEditor`].
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

#[derive(Debug, Clone)]
struct EditorContent {
    lines: Vec<RichLine>,
    current_line: usize,
    cursor_position: usize, // position within current line
}

impl Default for EditorContent {
    fn default() -> Self {
        Self {
            lines: vec![RichLine::new()],
            current_line: 0,
            cursor_position: 0,
        }
    }
}

impl EditorContent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(&self) -> String {
        self.lines.iter().map(|line| line.text()).collect::<Vec<_>>().join("\n")
    }

    pub fn insert_char(&mut self, c: char) {
        if self.current_line >= self.lines.len() {
            self.lines.push(RichLine::new());
        }

        let line = &mut self.lines[self.current_line];
        if line.spans.is_empty() {
            line.spans.push(RichSpan::default());
        }

        let span = &mut line.spans[0];
        span.text.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn insert_newline(&mut self) {
        let current_line = &mut self.lines[self.current_line];
        let remaining_text = current_line.spans[0].text.split_off(self.cursor_position);

        let new_line = RichLine::new();
        self.lines.insert(self.current_line + 1, new_line);
        self.lines[self.current_line + 1].spans[0].text = remaining_text;

        self.current_line += 1;
        self.cursor_position = 0;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            let line = &mut self.lines[self.current_line];
            line.spans[0].text.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        } else if self.current_line > 0 {
            // Join with previous line
            let current_text = self.lines[self.current_line].spans[0].text.clone();
            self.lines.remove(self.current_line);
            self.current_line -= 1;
            self.cursor_position = self.lines[self.current_line].spans[0].text.len();
            self.lines[self.current_line].spans[0].text.push_str(&current_text);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        } else if self.current_line > 0 {
            self.current_line -= 1;
            self.cursor_position = self.lines[self.current_line].spans[0].text.len();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.lines[self.current_line].spans[0].text.len();
        if self.cursor_position < line_len {
            self.cursor_position += 1;
        } else if self.current_line < self.lines.len() - 1 {
            self.current_line += 1;
            self.cursor_position = 0;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.current_line > 0 {
            self.current_line -= 1;
            let line_len = self.lines[self.current_line].spans[0].text.len();
            self.cursor_position = self.cursor_position.min(line_len);
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.current_line < self.lines.len() - 1 {
            self.current_line += 1;
            let line_len = self.lines[self.current_line].spans[0].text.len();
            self.cursor_position = self.cursor_position.min(line_len);
        }
    }

    pub fn toggle_bold_at_cursor(&mut self) {
        if self.current_line >= self.lines.len() {
            return;
        }

        let line = &mut self.lines[self.current_line];
        if line.spans.is_empty() {
            return;
        }

        // For simplicity, toggle bold on the current span
        // In a more advanced implementation, this would handle span splitting
        if let Some(span) = line.spans.get_mut(0) {
            span.bold = !span.bold;
        }
    }

    pub fn select_line(&mut self, line_index: usize) {
        for (i, line) in self.lines.iter_mut().enumerate() {
            line.selected = i == line_index;
        }
    }

    pub fn move_line_up(&mut self, line_index: usize) {
        if line_index > 0 {
            self.lines.swap(line_index, line_index - 1);
            if self.current_line == line_index {
                self.current_line -= 1;
            } else if self.current_line == line_index - 1 {
                self.current_line += 1;
            }
        }
    }

    pub fn move_line_down(&mut self, line_index: usize) {
        if line_index < self.lines.len() - 1 {
            self.lines.swap(line_index, line_index + 1);
            if self.current_line == line_index {
                self.current_line += 1;
            } else if self.current_line == line_index + 1 {
                self.current_line -= 1;
            }
        }
    }
}
