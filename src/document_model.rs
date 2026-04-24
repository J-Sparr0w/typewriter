use std::{cell::Cell, f32, time::Instant};

use iced::{
    Alignment, Border, Color, Element, Font, Length, Padding, Pixels, Point, Rectangle, Shadow, Size, Theme, Vector, advanced::{
        Layout, Text, Widget, input_method, layout, mouse, renderer::{self, Quad}, text::{Highlight, Paragraph, Wrapping, paragraph}, widget::{Tree, operation, tree}
    }, alignment, widget::{
        self, rich_text, text::{self, LineHeight, Span}, text_editor::{Binding, KeyPress}
    }
};

use ropey::{Rope, RopeSlice};

use crate::rich_text_editor::CursorOffset;

#[derive(Debug,Default, Copy, Clone, PartialEq)]
pub struct FontStyle{
    bold: bool,
    italic: bool
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Formatting {
    size: Option<Pixels>,
    line_height: Option<LineHeight>,
    font: Option<Font>,
    // making the text bold or italics take some more work
    pub font_style: FontStyle,
    color: Option<Color>,
    link: bool,
    highlight: Option<Highlight>,
    padding: Padding,
    underline: bool,
    strikethrough: bool,
}


#[derive(Debug, Default, Copy, Clone)]
pub struct RichTextSpan {
    pub start: CursorOffset,
    pub end: CursorOffset,
    pub formatting: Formatting,
}

#[derive(Debug, Default, Clone)]
pub struct LineFormat{
    pub size: Option<Pixels>,
    pub line_height: Option<LineHeight>,
    pub font: Option<iced::Font>,
    pub align_x: Option<text::Alignment>,
    pub align_y: Option<alignment::Vertical>,
    pub wrapping: Option<Wrapping>,
}

#[derive(Debug, Clone)]
pub struct Line {
    pub spans: Vec<RichTextSpan>,
    has_multiple_fonts: bool,
    pub format: LineFormat,
}

impl Line{
    pub fn needs_multiple_fonts(self: &Self) -> bool {
        if self.spans.is_empty() {
            return false
        }
        let mut iter =  self.spans.iter();
        let first_elem_style = iter.next().unwrap().formatting.font_style;
        iter.position(|s| s.formatting.font_style != first_elem_style).is_some()
    }

    pub fn text_span(&self) -> (usize, usize) {
        let start = self.spans.first().expect("Line must have atleast one span, I am trying the access the first span here").start;
        let end = self.spans.first().expect("Line must have atleast one span, I am trying the access the first span here").end;

        (start, end)
    }

    // pub fn add_span(&mut self, span: RichTextSpan){
    //     if self.spans.is_empty(){
    //         self.spans.push(span);
    //         return;
    //     }

    // }
}

impl Default for Line {
    fn default() -> Self {
        Self {
            spans: Vec::with_capacity(4),
            has_multiple_fonts: false,
            format: LineFormat::default(),
        }
    }
}

// impl<'a> Line<'a> {
//     pub fn to_paragraph(self: &Self) -> impl Paragraph {
//         let spans = self.spans.iter().map(|s| s.to_span());
//         let text = Text {
//             content: spans,
//             bounds: todo!(),
//             size: todo!(),
//             line_height: todo!(),
//             font: todo!(),
//             align_x: todo!(),
//             align_y: todo!(),
//             shaping: todo!(),
//             wrapping: todo!(),
//         };
//         Paragraph::with_spans(text);
//     }
// }

#[derive(Debug, Clone)]
pub struct Document {
    // Actual Text Content is stored as a rope
    pub text: Rope,
    pub lines: Vec<Line>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            text: Rope::new(),
            lines: Vec::with_capacity(128),
        }
    }
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn lines_ref(&self) -> &[Line]{
        &self.lines
    }
    pub fn content<'a>(&'a self, start: usize, end: usize) -> RopeSlice<'a> {
        self.text.slice(start..end)
    }
}
