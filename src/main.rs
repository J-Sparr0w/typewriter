use iced::Element;
use iced::widget::{TextEditor, text_editor};
use iced::widget::{button, column};

mod editor;

use editor::Editor;

type State = ();

enum Message {}

fn update(state: &mut State, message: Message) {}

fn view(state: &State) -> Element<Message> {
    let editor = Editor::new();
    editor.into()
}

fn main() {
    println!("Hello, world!");
    iced::run(update, view);
}
