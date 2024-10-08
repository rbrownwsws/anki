#[allow(warnings)]
mod bindings;

use bindings::anki::addon::host::log;
use bindings::exports::anki::addon::guest::{Guest, DeckId, Note};

struct Component;

impl Guest for Component {
    fn hello_guest() {
        log("Hello from a Wasm Addon!");
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
