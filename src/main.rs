use iced::Element;


// use rich_editor_widget::{Document,RichEditor};
// rich_editor_widget::CursorState
// use rich_text_editor::{RichTextEditor};

use typewriter::rich_text_editor::RichTextEditor;

type State = String;

#[derive(Debug)]
enum Message {}

fn update(state: &mut State, message: Message) {
    // match message {
    //     Message::TextChanged(text) => {
    //         *state = text;
    //         println!("Text changed: {}", state);
    //     }
    //     Message::LineSelected(line_index) => {
    //         println!("Line {} selected", line_index);
    //     }
    //     Message::LineMoved { from, to } => {
    //         println!("Line moved from {} to {}", from, to);
    //     }
    // }
}

fn view(state: &State) -> Element<Message> {
    let editor = RichTextEditor::new();
    // let editor = RichTextEditor::new()
    //     .on_change(|text| Message::TextChanged(text))
    //     .on_line_select(|line| Message::LineSelected(line))
    //     .on_line_move(|from, to| Message::LineMoved { from, to });
    // let editor = RichEditor::new(Document::from_str("Document Body"), CursorState::default() );

    editor.into()
}

fn main() {
    println!("Hello, world!");
    iced::run(update, view).unwrap();
}
