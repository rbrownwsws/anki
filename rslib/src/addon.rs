#![deny(unsafe_code)]
#![deny(rust_2024_compatibility)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

// Disable linting for generated code
#[allow(unsafe_code)]
#[allow(rust_2024_compatibility)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod bindings {
    wasmtime::component::bindgen!({
        path: "../addons/interface",
        world: "addon",
        // async: true,
    });
}

use std::error::Error;

use tracing::error;
use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, Linker};

use crate::decks::DeckId;
use crate::notes::Note;

use bindings::{Addon};

impl From<&Note> for bindings::anki::addon::notes::Note {
    fn from(value: &Note) -> Self {
        bindings::anki::addon::notes::Note {
            id: value.id.into(),
            guid: value.guid.clone(),
            note_type_id: value.notetype_id.into(),
            mtime: value.mtime.into(),
            usn: value.usn.into(),
            tags: value.tags.to_vec(),
            fields: value.fields().to_vec(),
            sort_field: value.sort_field.clone(),
            checksum: value.checksum,
        }
    }
}

impl From<&mut Note> for bindings::anki::addon::notes::Note {
    fn from(value: &mut Note) -> Self {
        (&*value).into()
    }
}

/// The information the AddonHost needs to provide Host functionality for a single Addon.
struct AddonHostState {

}

impl AddonHostState {
    pub fn new() -> Self {
        AddonHostState {

        }
    }
}

impl bindings::anki::addon::host::Host for AddonHostState {
    fn log(&mut self, message: String) {
        println!("{}", message);
    }
}

impl bindings::anki::addon::decks::Host for AddonHostState {

}

impl bindings::anki::addon::notes::Host for AddonHostState {

}

pub struct AddonMenuEntry {
    pub addon_id: u32,
    pub menu_idx: u32,
    pub label: String,
}

/// Everything we need to interact with an instance of an addon.
pub struct AddonContext {
    store: Store<AddonHostState>,
    instance: Addon,
    menu_entries: Vec<String>,
}

/// Manages interaction with Wasm Addons.
pub struct AddonHost {
    engine: Engine,
    addons: Vec<AddonContext>,
}

impl AddonHost {
    pub fn new() -> Result<AddonHost, Box<dyn Error>> {
        let mut config = Config::default();
        config.wasm_component_model(true);

        let engine = Engine::new(&config)?;

        Ok(AddonHost {
            engine,
            addons: Vec::new(),
        })
    }

    /// Load a new addon into the addon host
    pub fn load(&mut self, addon_bytes: impl AsRef<[u8]>) -> Result<(), Box<dyn Error>> {
        let component = Component::new(&self.engine, addon_bytes)?;

        // Set up the linker so it knows where to find Host methods
        let mut linker = Linker::new(&self.engine);
        Addon::add_to_linker(&mut linker, |state| state)?;

        // Create an instance of the addon
        let mut store = Store::new(&self.engine, AddonHostState::new());
        let instance = Addon::instantiate(&mut store, &component, &linker)?;

        // Initialise the addon
        let manifest = instance.anki_addon_guest().call_init(&mut store).unwrap();

        // Store the addon instance for later
        let context = AddonContext {
            store,
            instance,
            menu_entries: manifest.tool_menu_entries,
        };
        self.addons.push(context);

        Ok(())
    }

    /// Get rid of all loaded addons
    pub fn unload_all(&mut self) {
        self.addons.clear();
    }

    pub fn get_all_tool_menu_entries(&self) -> Vec<AddonMenuEntry> {
        let mut entries = Vec::new();

        for (addon_id, addon_context) in self.addons.iter().enumerate() {
            for (menu_idx, label) in addon_context.menu_entries.iter().enumerate() {
                entries.push(AddonMenuEntry {
                    addon_id: addon_id as u32,
                    menu_idx: menu_idx as u32,
                    label: label.clone(),
                });
            }
        }

        entries
    }
    
    pub fn on_tool_menu_entry_clicked(&mut self, addon_id: u32, menu_id: u32) {
        let addon_context = self.addons.get_mut(addon_id as usize).unwrap();
        
        addon_context.instance.anki_addon_guest()
            .call_on_tool_menu_entry_clicked(&mut addon_context.store, menu_id)
            .unwrap();
    }

    pub fn event_before_add_note(&mut self, note: &mut Note, did: DeckId) {
        for addon_context in &mut self.addons {
            let addon_note: bindings::anki::addon::notes::Note = note.into();
            
            let res = addon_context.instance.anki_addon_guest().call_before_add_note(
                &mut addon_context.store,
                &addon_note,
                did.into()
            );
            
            match res {
                Ok(Some(edited_note)) => {
                    note.id = edited_note.id.into();
                    note.guid = edited_note.guid;
                    note.notetype_id = edited_note.note_type_id.into();
                    note.mtime = edited_note.mtime.into();
                    note.usn = edited_note.usn.into();
                    note.tags = edited_note.tags;
                    
                    for (i, value) in edited_note.fields.iter().enumerate() {
                        note.set_field(i, value.as_str()).unwrap();
                    }
                    
                    note.sort_field = edited_note.sort_field;
                    note.checksum = edited_note.checksum;
                },
                Ok(None) => {/* Do nothing */},
                Err(e) => error!("Error calling addon: {:?}", e),
            }
        }
    }
}
