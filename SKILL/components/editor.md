---
description: Zopra Editor Component
---

# Editor

The `<Editor>` component wraps GPUI's high-performance native text editor, bringing it into Zopra's reactive `view!` macro. 
Because text buffers can hold megabytes of data, the text value itself is not passed as a declarative prop. Instead, you manage an `Entity<EditorState>` that tracks the cursor, scroll position, undo history, and text value persistently.

## Usage

First, initialize the editor state using the `use_editor` hook. Then, pass the state reference to the `<Editor>` component.
Because the generated component does not accept styles directly, you should wrap it in a `<div class="...">` to position and style it.

```rust
use zopra::{component, view, hooks::use_editor};
use zopra::components::Editor;

#[component]
pub fn MyEditor() {
    let editor_state = use_editor(window, cx);

    // Imperatively set the initial value
    if editor_state.read(cx).value().is_empty() {
        editor_state.update(cx, |ed, cx| {
            ed.set_value("fn main() {\n    println!(\"Hello, Zopra!\");\n}", window, cx);
        });
    }

    view! {
        // Wrap the Editor in a styled container
        <div class="w-full h-96 border border-zinc-700 bg-zinc-900 font-mono text-sm">
            <Editor 
                state={&editor_state}
                language="rust"
                line_numbers={true}
                folding={true}
                auto_close={true}
                smart_indent={true}
                readonly={false}
                searchable={true}
            />
        </div>
    }
}
```

## Props

The following props are natively supported and reactive. If their values change, Zopra will automatically update the underlying `EditorState`.

- `state: &Entity<EditorState>` - **(Required)** The persistent state entity powering the editor.
- `language: Option<&'static str>` - Enables syntax highlighting (e.g., `"rust"`, `"json"`, `"http"`). Requires the language grammar to be registered globally via `set_language_config` or `LanguageRegistry`.
- `line_numbers: Option<bool>` - Toggles the left-hand line number gutter.
- `folding: Option<bool>` - Enables code folding brackets.
- `auto_close: Option<bool>` - Automatically closes brackets like `{}`, `()`, and `""`.
- `smart_indent: Option<bool>` - Automatically indents new lines based on the language's grammar rules.
- `readonly: Option<bool>` - Prevents the user from modifying the text while preserving selection and search capabilities.
- `searchable: Option<bool>` - Enables the built-in Ctrl+F / Cmd+F search panel.
- `show_whitespaces: Option<bool>` - Renders visible dots for spaces and arrows for tabs.

## Getting and Setting Text

Text is managed imperatively through the `EditorState`:

```rust
// Read the text
let text = editor_state.read(cx).value();

// Write the text
editor_state.update(cx, |ed, cx| {
    ed.set_value("new text", window, cx);
});
```
