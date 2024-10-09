#[allow(warnings)]
mod bindings;

use bindings::anki::addon::host::log;
use bindings::exports::anki::addon::guest::{Guest, AddonManifest, DeckId, Note};

struct ToolMenuEntry {
    label: &'static str,
    callback: fn()
}

struct Component;

const TOOL_MENU_ENTRIES: &[ToolMenuEntry] = &[
    ToolMenuEntry {
        label: "Say Hello",
        callback: say_hello,
    },
    ToolMenuEntry {
        label: "Say Goodbye",
        callback: say_goodbye,
    }
];

fn say_hello() {
    log("Hello from a Wasm Addon!");
}

fn say_goodbye() {
    log("Goodbye from a Wasm Addon!");
}

impl Guest for Component {
    fn init() -> AddonManifest {
        say_hello();
        
        let tool_menu_entries = TOOL_MENU_ENTRIES.iter().map(|entry| entry.label.to_owned()).collect();

        AddonManifest {
            name: "Example Addon".to_string(),
            tool_menu_entries,
        }
    }

    fn on_tool_menu_entry_clicked(idx: u32) {
        if let Some(entry) = TOOL_MENU_ENTRIES.get(idx as usize) {
            (entry.callback)();
        }
    }


    fn before_add_note(note: Note, did: DeckId) -> Option<Note> {
        log(&format!("Before add note: DID - {}, Note GUID - {}, Note Fields - {:?}", did, note.guid, note.fields));
        
        let mut edited_note = note.clone();
        
        for field in &mut edited_note.fields {
            field.push_str(" [Hello from a Wasm addon!]");
        }
        
        Some(edited_note)
    }
}

bindings::export!(Component with_types_in bindings);
