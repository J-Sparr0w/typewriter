# Typewriter: MS Word-like Rich Text Editor

A powerful rich text editor built with Rust and Iced, featuring line-level control and MS Word-like functionality.

### WIP: Just Started and do not work on it that often. But would like to get atleast basic functionality up and running as 

## Features

### Rich Text Editing
- **Multi-line text editing** with proper line management
- **Cursor navigation** with arrow keys (left, right, up, down)
- **Text insertion and deletion** with backspace
- **Enter key support** for creating new lines- **Bold text formatting** with Ctrl+B to toggle bold on/off
### Line-Level Control
- **Line selection** by clicking on any line
- **Visual line highlighting** with light blue background for selected lines
- **Line movement** using Alt+PageUp/PageDown keys:
  - `Alt + PageUp`: Move current line up
  - `Alt + PageDown`: Move current line down

### User Interface
- **Clean, minimal design** with white background
- **Real-time cursor** showing current editing position
- **Responsive layout** that fills the available space
- **GPU-accelerated rendering** for smooth performance

## Keyboard Shortcuts

### Text Editing
- **Arrow Keys**: Navigate cursor (left, right, up, down)
- **Backspace**: Delete character before cursor
- **Enter**: Create new line
- **Ctrl + Arrow Keys**: Quick navigation between lines
- **Ctrl + B**: Toggle bold formatting on current line

### Line Operations
- **Alt + PageUp**: Move current line up
- **Alt + PageDown**: Move current line down
- **Mouse Click**: Select a line

## Technical Details

Built with:
- **Rust** for performance and memory safety
- **Iced** GUI framework for cross-platform compatibility
- **Custom widget implementation** for rich text editing capabilities

## Usage

```bash
cargo run
```

The editor will open in a window where you can start typing immediately. Click on any line to select it, and use the keyboard shortcuts to manipulate lines.
