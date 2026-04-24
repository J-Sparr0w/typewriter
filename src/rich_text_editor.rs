use std::{cell::Cell, f32, time::Instant};

use iced::{
    advanced::{
        input_method, layout, mouse,
        renderer::{self, Quad},
        text::{paragraph, Highlight, Paragraph, Wrapping},
        widget::{operation, tree, Tree},
        Layout, Text, Widget,
    },
    alignment,
    font::Weight,
    widget::{
        self, rich_text,
        text::{self, LineHeight, Span},
        text_editor::{Binding, KeyPress},
    },
    Alignment, Border, Color, Element, Font, Length, Padding, Pixels, Point, Rectangle, Shadow,
    Size, Theme, Vector,
};

use ropey::{Rope, RopeSlice};

use crate::document_model::{Document, Formatting, Line, RichTextSpan};

// pub enum
//

#[derive(Debug, Default, Clone)]
struct RenderLine {
    // index of the starting segment of line
    start: usize,
    // index of the ending segment of line
    end: usize,
}

#[derive(Debug, Default, Clone)]
struct RenderParagraph<P: Paragraph<Font = iced::Font>> {
    // A segment is just a Renderer::Paragraph
    segments: Vec<P>,
    // A line can have 1 or more segments
    // When a new line starts, a new segment is created
    lines: Vec<RenderLine>,
}


impl<P: Paragraph<Font = iced::Font>> RenderParagraph<P> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_segment(&mut self, line_idx: usize, paragraph: P) {
        if let Some(line) = self.lines.get_mut(line_idx) {
            // TODO: This is just dummy code, this probably displays text in the opposite order
            self.segments.insert(line_idx, paragraph);
            line.end += 1;
        } else {
            self.segments.push(paragraph);
            self.lines.push(RenderLine {
                start: line_idx,
                end: line_idx,
            });
        }
    }
}

fn build_paragraphs<P: Paragraph<Font = iced::Font>, Renderer>(
    doc: &Document,
    renderer: &Renderer,
    bounds: Size<f32>,
) -> RenderParagraph<P>
where
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    let lines: &[Line] = doc.lines_ref();
    let mut render_paragraph = RenderParagraph::new();

    let default_align_x = text::Alignment::Default;
    let default_align_y = alignment::Vertical::Center;
    let default_shaping = text::Shaping::Advanced;
    let default_wrapping = text::Wrapping::default();

    for (i, line) in lines.iter().enumerate() {
        let line_idx = i;

        if line.needs_multiple_fonts() {
            let mut iter = line.spans.iter();
            let first = iter
                .next()
                .expect("Calling next span won't cause panic cause it is checked elsewhere");
            let prev_font = first.formatting.font_style;

            let mut start = first.start;
            let mut end = first.end;
            let last_idx = line.spans.len() - 1;

            for (i, span) in iter.enumerate() {
                let curr_font = span.formatting.font_style;
                if curr_font == prev_font {
                    end = span.end;
                } else {
                    // create a paragraph and add it to cache and start a new one
                    let content = doc.content(start, end).to_string();
                    let text: Text<&str, iced::Font> = Text {
                        content: &content,
                        bounds: bounds,
                        size: line.format.size.unwrap_or_else(|| renderer.default_size()),
                        line_height: line
                            .format
                            .line_height
                            .unwrap_or_else(|| LineHeight::default()),
                        font: line.format.font.unwrap_or_else(|| renderer.default_font()),
                        align_x: line.format.align_x.unwrap_or(text::Alignment::Default),
                        align_y: line.format.align_y.unwrap_or(default_align_y),
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::default(),
                    };
                    render_paragraph.add_segment(line_idx, Paragraph::with_text(text));

                    start = end;
                    end = span.end
                }

                if i == last_idx {
                    let content = doc.content(start, end).to_string();
                    let text: Text<&str, iced::Font> = Text {
                        content: &content,
                        bounds: bounds,
                        size: line.format.size.unwrap_or_else(|| renderer.default_size()),
                        line_height: line
                            .format
                            .line_height
                            .unwrap_or_else(|| LineHeight::default()),
                        font: line.format.font.unwrap_or_else(|| renderer.default_font()),
                        align_x: line.format.align_x.unwrap_or(text::Alignment::Default),
                        align_y: line.format.align_y.unwrap_or(default_align_y),
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::default(),
                    };
                    render_paragraph.add_segment(line_idx, Paragraph::with_text(text));
                }
            }
        } else {
            let (start, end) = line.text_span();
            let content = doc.content(start, end).to_string();
            let text: Text<&str, iced::Font> = Text {
                content: &content,
                bounds: bounds,
                size: line.format.size.unwrap_or_else(|| renderer.default_size()),
                line_height: line
                    .format
                    .line_height
                    .unwrap_or_else(|| LineHeight::default()),
                font: line.format.font.unwrap_or_else(|| renderer.default_font()),
                align_x: line.format.align_x.unwrap_or(text::Alignment::Default),
                align_y: line.format.align_y.unwrap_or(default_align_y),
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::default(),
            };
            render_paragraph.add_segment(line_idx, Paragraph::with_text(text));
        }
    }

    render_paragraph
}

enum Action {
    Insert(char),
    DeleteBack,
    DeleteForward,
}

pub struct RichTextEditor<'a, Message> {
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
}

impl<'a, Message> RichTextEditor<'a, Message> {
    pub fn new() -> Self {
        Self {
            id: None,
            font: Some(iced::Font::DEFAULT),
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
        }
    }
}

impl<'a, Message, Renderer> From<RichTextEditor<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn from(editor: RichTextEditor<'a, Message>) -> Self {
        Self::new(editor)
    }
}

#[derive(Debug, Default, Clone)]
pub struct EditorState<P: Paragraph<Font = iced::Font>> {
    // is_focused: Option<Focus>,
    // is_dragging: Option<Drag>,
    // is_pasting: Option<Value>,
    // preedit: Option<input_method::Preedit>,
    last_click: Option<mouse::Click>,
    cursor: Cursor,
    document: Document,
    requires_redraw: Cell<bool>,
    render_cache: RenderParagraph<P>,
    // last_span_start: usize,
    // last_span: RichTextSpan,
    // keyboard_modifiers: keyboard::Modifiers,
    // TODO: Add stateful horizontal scrolling offset
}

impl<'a, Message, Renderer> Widget<Message, Theme, Renderer> for RichTextEditor<'a, Message>
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
        tree::Tag::of::<EditorState<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(EditorState::<Renderer::Paragraph>::new())
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree
            .state
            .downcast_mut::<EditorState<Renderer::Paragraph>>();
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
        let padding = self.padding.fit(Size::ZERO, limits.max());
        let height = self.line_height.to_absolute(text_size);

        let limits = limits.width(self.width).shrink(padding);
        let text_bounds = limits.resolve(self.width, height, Size::ZERO);

        println!("Layout function in action");

        state.render_cache = build_paragraphs(&state.document, renderer, text_bounds);

        // let spans: Vec<RichTextSpan> = state.document.lines.iter().flat_map(|l| l.spans.iter()).map(|&s| s).collect();
        // dbg!(&state.document);
        // let to_span = |s:&RichTextSpan| -> Span {
        //     let text = state.document.text.slice(s.start..s.end);
        //         // .slice
        //         // .expect("HOW DID THE SLICE GET EMPTY")
        //         // .as_str()
        //         // .expect("Converting a rope slice to a &str is the culprit");
        //     Span {
        //         text: text.into(),
        //         size: s.size,
        //         line_height: s.line_height,
        //         font: s.font,
        //         color: s.color,
        //         link: s.link,
        //         highlight: s.highlight,
        //         padding: s.padding,
        //         underline: s.underline,
        //         strikethrough: s.strikethrough,
        //     }
        // };
        // let text_content = spans.iter().map(|s| to_span(s)).collect::<Vec<Span>>();
        // let text = Text {
        //     content: text_content.as_ref(),
        //     bounds: text_bounds,
        //     size: text_size,
        //     line_height: self.line_height,
        //     font: self.font.unwrap_or(iced::Font::DEFAULT),
        //     align_x: text::Alignment::Default,
        //     align_y: alignment::Vertical::Center,
        //     shaping: text::Shaping::Advanced,
        //     wrapping: text::Wrapping::default(),
        // };
        // state.paragraph = Paragraph::with_spans(text);
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
        let state = tree
            .state
            .downcast_ref::<EditorState<Renderer::Paragraph>>();
        // let value = "Value String not placeholder";
        // // let is_disabled = self.on_input.is_none();

        if state.requires_redraw.replace(false) {
            println!("Going to draw cuz values changed");
        }

        let bounds = layout.bounds();
        // dbg!(bounds);
        let mut children_layout = layout.children();
        let text_bounds = children_layout.next().unwrap().bounds();

        // // let style = theme.style(&self.class, self.last_status.unwrap_or(Status::Disabled));

        renderer.fill_quad(
            renderer::Quad {
                bounds: *viewport,
                // border: Bord,
                ..renderer::Quad::default()
            },
            Color::WHITE,
        );

        // let text = state.document.text.to_string(); //value.to_string();
        for (i, paragraph) in state.render_cache.segments.iter().enumerate() {
            let viewport = text_bounds;

            // let draw = |renderer: &mut Renderer, viewport| {
            // };

            let alignment_offset =
                alignment_offset(text_bounds.width, paragraph.min_width(), self.alignment);
            let  position =
                text_bounds.anchor(paragraph.min_bounds(), Alignment::Start, Alignment::Center)
                    + Vector::new(alignment_offset - 0.0, 0.0 );
            // dbg!(position);
            renderer.with_translation(Vector::ZERO, |_| {});

            renderer.fill_paragraph(paragraph, position, Color::BLACK, viewport);
        }

        // let text = Text {
        //     content: self.placeholder.as_str(),
        //     bounds: layout.bounds().size(),
        //     size: self.text_size.unwrap_or_else(|| renderer.default_size()),
        //     line_height: self.line_height,
        //     font: self.font.unwrap_or_else(|| renderer.default_font()),
        //     align_x: text::Alignment::Default,
        //     align_y: alignment::Vertical::Center,
        //     shaping: text::Shaping::Advanced,
        //     wrapping: text::Wrapping::default(),
        // };

        // let text = Text {
        //     content: state
        //         .content
        //         .text
        //         .slice(..)
        //         .as_str()
        //         .unwrap_or("Default Text")
        //         .to_owned(),
        //     bounds: layout.bounds().size(),
        //     size: self.text_size.unwrap_or_else(|| renderer.default_size()),
        //     line_height: self.line_height,
        //     font: self.font.unwrap_or_else(|| renderer.default_font()),
        //     align_x: text::Alignment::Default,
        //     align_y: alignment::Vertical::Center,
        //     shaping: text::Shaping::Advanced,
        //     wrapping: text::Wrapping::default(),
        // };

        // renderer.fill_text(text, Point { x: 10., y: 10. }, Color::WHITE, *viewport);
        println!("DID IT PRINT?????????")
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::Text
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        match event {
            iced::Event::Keyboard(event) => {
                let state = tree
                    .state
                    .downcast_mut::<EditorState<Renderer::Paragraph>>();

                match event {
                    iced::keyboard::Event::KeyPressed {
                        key,
                        modified_key,
                        physical_key,
                        location,
                        modifiers,
                        text,
                        repeat,
                    } => {
                        // iced::widget::rich_text(spans);

                        if let Some(text) = text {
                            if let Some(c) = text.chars().next().filter(|c| !c.is_control()) {
                                state.insert(c);
                                state.update_render_caches(renderer, layout);

                                dbg!(c);

                                shell.capture_event();

                                // focus.updated_at = Instant::now();
                                // replace_paragraph(
                                //     renderer,
                                //     state,
                                //     layout,
                                //     self.font,
                                //     self.text_size,
                                //     self.line_height,
                                // );
                                shell.request_redraw();
                                return;
                            }
                        }
                    }
                    iced::keyboard::Event::KeyReleased {
                        key,
                        modified_key,
                        physical_key,
                        location,
                        modifiers,
                    } => {}

                    iced::keyboard::Event::ModifiersChanged(modifiers) => {}
                }
            }
            // iced::Event::Mouse(event) => todo!(),
            // iced::Event::Window(event) => todo!(),
            // iced::Event::Touch(event) => todo!(),
            // iced::Event::InputMethod(event) => todo!(),
            _ => {}
        }
    }
}

// impl<P: Paragraph> operation::TextInput for EditorState<P> {
//     fn text(&self) -> &str {
//         self.content.text.slice(0..).as_str().unwrap_or("")
//     }

//     fn move_cursor_to_front(&mut self) {
//         // self.cursor.move_to(0);
//     }

//     fn move_cursor_to_end(&mut self) {
//         // self.cursor.move_to(usize::MAX);
//     }

//     fn move_cursor_to(&mut self, position: usize) {
//         // self.cursor.move_to(position);
//     }

//     fn select_all(&mut self) {
//         // self.cursor.select_range(0, usize::MAX);
//     }

//     fn select_range(&mut self, start: usize, end: usize) {
//         // self.cursor.select_range(start, end);
//     }
// }

// fn measure_cursor_and_scroll_offset(
//     paragraph: &impl Paragraph,
//     text_bounds: Rectangle,
//     cursor_index: usize,
// ) -> (f32, f32) {
//     let grapheme_position = paragraph
//         .grapheme_position(0, cursor_index)
//         .unwrap_or(Point::ORIGIN);

//     let offset = ((grapheme_position.x + 5.0) - text_bounds.width).max(0.0);
// }

// fn replace_paragraph<P: Paragraph<Font = iced::Font>, Renderer>(
//     renderer: &Renderer,
//     state: &mut EditorState<P>,
//     layout: Layout<'_>,
//     font: Option<iced::Font>,
//     text_size: Option<Pixels>,
//     line_height: LineHeight,
// ) where
//     Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
// {
//     let font = font.unwrap_or(iced::Font::DEFAULT);
//     let text_size = text_size.unwrap_or_else(|| renderer.default_size());

//     let mut children_layout = layout.children();
//     let text_bounds = children_layout.next().unwrap().bounds();

//     let spans: Vec<RichTextSpan> = state
//         .document
//         .lines
//         .iter()
//         .flat_map(|l| l.spans.iter())
//         .map(|&s| s)
//         .collect();
//     dbg!(&state.document);
//     let to_span = |s: &RichTextSpan| -> Span {
//         let text = state.document.text.slice(s.start..s.end);
//         // .slice
//         // .expect("HOW DID THE SLICE GET EMPTY")
//         // .as_str()
//         // .expect("Converting a rope slice to a &str is the culprit");
//         Span {
//             text: text.into(),
//             size: s.size,
//             line_height: s.line_height,
//             font: s.font,
//             color: s.color,
//             link: s.link,
//             highlight: s.highlight,
//             padding: s.padding,
//             underline: s.underline,
//             strikethrough: s.strikethrough,
//         }
//     };

//     let text_content = spans.iter().map(|s| to_span(s)).collect::<Vec<Span>>();
//     let text = Text {
//         content: text_content.as_ref(),
//         bounds: text_bounds.size(),
//         size: text_size,
//         line_height: line_height,
//         font: font,
//         align_x: text::Alignment::Default,
//         align_y: alignment::Vertical::Center,
//         shaping: text::Shaping::Advanced,
//         wrapping: text::Wrapping::default(),
//     };
//     state.paragraph = Paragraph::with_spans(text);
// }

pub type CursorOffset = usize;

#[derive(Debug, Clone)]
pub enum Cursor {
    Index(CursorOffset),
    Selection(CursorOffset, CursorOffset),
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

impl<P: Paragraph<Font = iced::Font>> EditorState<P> {
    /// Creates a new [`EditorState`], representing an unfocused [`TextInput`].
    pub fn new() -> Self {
        Self {
            requires_redraw: Cell::new(true),
            ..Default::default()
        }
    }

    pub fn insert(self: &mut Self, ch: char) {
        let cursor_pos = self.get_cursor_pos();
        self.requires_redraw = Cell::new(true);

        if self.document.lines.is_empty() {
            let mut line = Line::default();
            let span = RichTextSpan {
                start: 0,
                end: 1,
                // size: None,
                // line_height: None,
                // font: None,
                // color: None,
                // link: None,
                // highlight: None,
                // padding: Padding::default(),
                // underline: false,
                // strikethrough: false,
                formatting: Formatting::default(),
            };
            self.document.text.insert_char(cursor_pos, ch);
            //then update the span
            line.spans.push(span);
            self.document.lines.push(line);
            // must be last, if there is an error before inserting, we should not update the cursor pos.
            self.cursor.inc();
            return;
        }

        let (line, col) = {
            let start_time = Instant::now();

            let line_idx = self.document.text.char_to_line(cursor_pos);
            let line_start_offset = self.document.text.line_to_char(line_idx);

            let col = cursor_pos - line_start_offset;
            let line = self.document.lines.get_mut(line_idx).expect(&format!("Accessing line at index = {}, should be possible. This line should be present as it is present in the rope.", line_idx));

            let elapsed = start_time.elapsed();
            println!(
                "Time took for converting cursor offset from document to lines and cols => {:?}",
                elapsed
            );

            (line, col)
        };
        println!("CursorPos To Find: {}", cursor_pos);
        let span = match line
            .spans
            .iter()
            .position(|s| cursor_pos >= s.start && cursor_pos <= s.end + 1)
        {
            Some(idx) => {
                println!("Span found at index: {}", idx);
                //then update the span
                // span.end = col + 1;
                line.spans[idx].end += 1;

                // insert to doc
                self.document.text.insert_char(cursor_pos, ch);

                // span.slice = Some(self.document.text.slice(span.start..span.end));

                // must be last, if there is an error before inserting, we should not update the cursor pos.
                self.cursor.inc();
            }
            None => {
                unreachable!();
                // let span = RichTextSpan {
                //     start: col,
                //     end: col,
                //     size: None,
                //     line_height: None,
                //     font: None,
                //     color: None,
                //     link: None,
                //     highlight: None,
                //     padding: Padding::default(),
                //     underline: false,
                //     strikethrough: false,
                // };
                // line.spans.push(span);
                // span
            }
        };
    }

    fn update_render_caches<Renderer>(&mut self, renderer: &Renderer, layout: Layout<'_>)
    where
        Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
    {
        dbg!(&self.document);
        dbg!(&self.render_cache.lines);
        dbg!(&self.render_cache.segments.len());
        let cursor_pos = self.get_cursor_pos();
        let line_idx = self.document.text.char_to_line(cursor_pos);
        let line = self.document.lines.get(line_idx).expect(&format!("Accessing line at index = {}, should be possible. This line should be present as it is present in the rope.", line_idx));

        let bounds = layout.bounds().size();

        if let Some(line) = self.render_cache.lines.get(line_idx){
            let start = line.start;
            let end = line.end;
            println!("Removing Segments from indices: {} to {}", start, end);
            for i in start..=end {
                self.render_cache.segments.remove(i);
            }
            self.render_cache.lines.remove(line_idx);
        }

        if line.needs_multiple_fonts() {
            let mut iter = line.spans.iter();
            let first = iter
                .next()
                .expect("Calling next span won't cause panic cause it is checked elsewhere");
            let prev_font = first.formatting.font_style;

            let mut start = first.start;
            let mut end = first.end;
            let last_idx = line.spans.len() - 1;

            for (i, span) in iter.enumerate() {
                let curr_font = span.formatting.font_style;
                if curr_font == prev_font {
                    end = span.end;
                } else {
                    // create a paragraph and add it to cache and start a new one
                    let content = self.document.content(start, end).to_string();
                    let text: Text<&str, iced::Font> = Text {
                        content: &content,
                        bounds: bounds,
                        size: line.format.size.unwrap_or_else(|| renderer.default_size()),
                        line_height: line
                            .format
                            .line_height
                            .unwrap_or_else(|| LineHeight::default()),
                        font: line.format.font.unwrap_or_else(|| {
                            let mut font = renderer.default_font();
                            font.weight = Weight::Bold;
                            font
                        }),
                        align_x: line.format.align_x.unwrap_or(text::Alignment::Default),
                        align_y: line.format.align_y.unwrap_or(alignment::Vertical::Center),
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::default(),
                    };

                    self.render_cache
                        .add_segment(line_idx, Paragraph::with_text(text));

                    start = end;
                    end = span.end
                }

                if i == last_idx {
                    let content = self.document.content(start, end).to_string();
                    let text: Text<&str, iced::Font> = Text {
                        content: &content,
                        bounds: bounds,
                        size: line.format.size.unwrap_or_else(|| renderer.default_size()),
                        line_height: line
                            .format
                            .line_height
                            .unwrap_or_else(|| LineHeight::default()),
                        font: line.format.font.unwrap_or_else(|| renderer.default_font()),
                        align_x: line.format.align_x.unwrap_or(text::Alignment::Default),
                        align_y: line.format.align_y.unwrap_or(alignment::Vertical::Center),
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::default(),
                    };
                    self.render_cache
                        .add_segment(line_idx, Paragraph::with_text(text));
                }
            }
        } else {
            let (start, end) = line.text_span();
            let content = self.document.content(start, end).to_string();
            let text: Text<&str, iced::Font> = Text {
                content: &content,
                bounds: bounds,
                size: line.format.size.unwrap_or_else(|| renderer.default_size()),
                line_height: line
                    .format
                    .line_height
                    .unwrap_or_else(|| LineHeight::default()),
                font: line.format.font.unwrap_or_else(|| renderer.default_font()),
                align_x: line.format.align_x.unwrap_or(text::Alignment::Default),
                align_y: line.format.align_y.unwrap_or(alignment::Vertical::Center),
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::default(),
            };

            println!("Adding New Segments ");
            self.render_cache
                .add_segment(line_idx, Paragraph::with_text(text));
        }
    }

    pub fn get_cursor_pos(self: &Self) -> CursorOffset {
        match self.cursor {
            Cursor::Index(pos) => pos,
            Cursor::Selection(start, end) => {
                println!(
                    "You asked for cursor_pos while there is a selection, so giving you the end of the selection"
                );
                end
            }
        }
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
