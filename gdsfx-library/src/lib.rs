use std::{path::Path, collections::HashMap, time::Duration};

use anyhow::Result;

mod requests;
mod parse;

mod files;
pub use files::FileEntry;

pub type EntryId = u32;

#[derive(Debug, Clone)]
pub struct Library {
    root_id: EntryId,
    sound_ids: Vec<EntryId>,

    entries: HashMap<EntryId, LibraryEntry>,
    child_map: HashMap<EntryId, Vec<EntryId>>,

    total_bytes: i64,
    total_duration: Duration,

    credits: Vec<Credit>,
}

#[derive(Debug, Clone)]
pub struct LibraryEntry {
    pub id: EntryId,
    pub name: String,
    pub parent_id: EntryId,
    pub kind: EntryKind,
}

#[derive(Debug, Clone)]
pub enum EntryKind {
    Category,
    Sound { bytes: i64, duration: Duration },
}

#[derive(Debug, Clone)]
pub struct Credit {
    pub name: String,
    pub link: String,
}

impl Library {
    pub fn load(gd_folder: impl AsRef<Path>) -> Self {
        const SFX_LIBRARY_FILE: &str = "sfxlibrary.dat";
    
        let file = gd_folder.as_ref().join(SFX_LIBRARY_FILE);
    
        gdsfx_files::read_file(&file).ok()
            .map(parse::parse_library_from_bytes)
            .filter(Self::check_library_version)
            .unwrap_or_else(|| {
                let bytes = requests::request_file(SFX_LIBRARY_FILE)
                    .and_then(|response| Ok(response.bytes()?))
                    .map(|bytes| bytes.to_vec())
                    .expect("Couldn't get library data");

                let _ = gdsfx_files::write_file(&file, &bytes);
                parse::parse_library_from_bytes(bytes)
            })
    }

    fn check_library_version(library: &Library) -> bool {
        const SFX_VERSION_ENDPOINT: &str = "sfxlibrary_version.txt";

        requests::request_file(SFX_VERSION_ENDPOINT).ok()
            .and_then(|response| response.text().ok())
            .map(|version| version == library.get_version())
            .unwrap_or(false)
    }

    pub fn get_root(&self) -> &LibraryEntry {
        self.entries.get(&self.root_id).unwrap()
    }

    pub fn iter_children(&self, entry: &LibraryEntry) -> impl Iterator<Item = &LibraryEntry> {
        self.child_map
            .get(&entry.id)
            .into_iter()
            .flatten()
            .flat_map(|id| self.entries.get(id))
    }

    pub fn iter_sounds(&self) -> impl Iterator<Item = &LibraryEntry> {
        self.sound_ids.iter().flat_map(|id| self.entries.get(id))
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn total_bytes(&self) -> i64 {
        self.total_bytes
    }

    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }

    pub fn get_version(&self) -> &str {
        &self.get_root().name
    }

    pub fn get_credits(&self) -> &Vec<Credit> {
        &self.credits
    }
}
