// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::path::PathBuf;
use std::sync::MutexGuard;

use anki_proto::generic;
use tracing::error;

use super::Backend;
use crate::collection::CollectionBuilder;
use crate::prelude::*;
use crate::progress::progress_to_proto;
use crate::services::BackendCollectionService;
use crate::storage::SchemaVersion;

impl BackendCollectionService for Backend {
    fn open_collection(&self, input: anki_proto::collection::OpenCollectionRequest) -> Result<()> {
        let mut guard = self.lock_closed_collection()?;

        let mut builder = CollectionBuilder::new(input.collection_path);
        builder
            .set_media_paths(input.media_folder_path, input.media_db_path)
            .set_server(self.server)
            .set_tr(self.tr.clone())
            .set_shared_progress_state(self.progress_state.clone())
            .set_addon_host(self.addon_host.clone());

        *guard = Some(builder.build()?);

        Ok(())
    }

    fn close_collection(
        &self,
        input: anki_proto::collection::CloseCollectionRequest,
    ) -> Result<()> {
        let desired_version = if input.downgrade_to_schema11 {
            Some(SchemaVersion::V11)
        } else {
            None
        };

        self.abort_media_sync_and_wait();
        let mut guard = self.lock_open_collection()?;
        let col_inner = guard.take().unwrap();

        if let Err(e) = col_inner.close(desired_version) {
            error!(" failed: {:?}", e);
        }

        Ok(())
    }

    fn create_backup(
        &self,
        input: anki_proto::collection::CreateBackupRequest,
    ) -> Result<generic::Bool> {
        // lock collection
        let mut col_lock = self.lock_open_collection()?;
        let col = col_lock.as_mut().unwrap();
        // await any previous backup first
        let mut task_lock = self.backup_task.lock().unwrap();
        if let Some(task) = task_lock.take() {
            task.join().unwrap()?;
        }
        // start the new backup
        let created = if let Some(task) = col.maybe_backup(input.backup_folder, input.force)? {
            if input.wait_for_completion {
                drop(col_lock);
                task.join().unwrap()?;
            } else {
                *task_lock = Some(task);
            }
            true
        } else {
            false
        };
        Ok(created.into())
    }

    fn await_backup_completion(&self) -> Result<()> {
        self.await_backup_completion()?;
        Ok(())
    }

    fn latest_progress(&self) -> Result<anki_proto::collection::Progress> {
        let progress = self.progress_state.lock().unwrap().last_progress;
        Ok(progress_to_proto(progress, &self.tr))
    }

    fn set_wants_abort(&self) -> Result<()> {
        self.progress_state.lock().unwrap().want_abort = true;
        Ok(())
    }

    fn init_addons(&self, request: anki_proto::collection::InitAddonsRequest) -> Result<()> {
        // Find the addons directory
        let addons_dir_path = PathBuf::from(request.addons_path);
        
        println!("addons_dir_path: {:?}", addons_dir_path);

        // Find addon files
        let addon_paths = std::fs::read_dir(&addons_dir_path)?
            // Get rid of entries we do not have permission to read etc.
            .filter_map(|x| x.ok())
            // Convert entries to paths
            .map(|x| x.path())
            // Filter out files that have the extension "wasm"
            .filter(|x| x.is_file() && x.extension().is_some_and(|extension| extension == "wasm"));

        let mut addon_host = self.addon_host.lock().expect("failed to obtain lock on addon_host");

        // Unload old addons
        addon_host.unload_all();

        // Load the addon files
        for path in addon_paths {
            println!("Loading addon at: {:?}", path);
            let addon_bytes = std::fs::read(path)?;

            addon_host.load(addon_bytes).expect("failed to load addon");
        }
        
        Ok(())
    }

    fn get_addon_tool_menus_entries(&self) -> Result<anki_proto::collection::AddonToolMenus> {
        let addon_host = self.addon_host.lock().expect("failed to obtain lock on addon_host");
        
        let mut entries = vec![];
        
        for entry in addon_host.get_all_tool_menu_entries() {
            entries.push(anki_proto::collection::AddonToolMenuEntry {
                menu_id: Some(anki_proto::collection::AddonMenuId {
                    addon_id: entry.addon_id,
                    menu_idx: entry.menu_idx,
                }),
                label: entry.label,
            });
        }
        
        Ok(anki_proto::collection::AddonToolMenus {
            entries
        })
    }

    fn on_click_addon_menu(&self, request: anki_proto::collection::AddonMenuId) -> Result<()> {
        let mut addon_host = self.addon_host.lock().expect("failed to obtain lock on addon_host");
        
        addon_host.on_tool_menu_entry_clicked(request.addon_id, request.menu_idx);
        
        Ok(())
    }
}

impl Backend {
    pub(super) fn lock_open_collection(&self) -> Result<MutexGuard<Option<Collection>>> {
        let guard = self.col.lock().unwrap();
        guard
            .is_some()
            .then_some(guard)
            .ok_or(AnkiError::CollectionNotOpen)
    }

    pub(super) fn lock_closed_collection(&self) -> Result<MutexGuard<Option<Collection>>> {
        let guard = self.col.lock().unwrap();
        guard
            .is_none()
            .then_some(guard)
            .ok_or(AnkiError::CollectionAlreadyOpen)
    }

    fn await_backup_completion(&self) -> Result<()> {
        if let Some(task) = self.backup_task.lock().unwrap().take() {
            task.join().unwrap()?;
        }
        Ok(())
    }
}
