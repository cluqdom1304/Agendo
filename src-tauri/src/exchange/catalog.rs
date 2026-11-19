//! What an import needs from the database: folders, tags, notes and stored images.

use super::*;

/// Finds or creates the folders and tags an import needs, remembering what it made.
pub(super) struct Catalog {
    pub(super) folders: HashMap<(Option<String>, String), String>,
    pub(super) tags: HashMap<String, String>,
    pub(super) new_folders: usize,
    pub(super) new_tags: usize,
    /// Origin and content hash of the notes imported before, and of those imported now.
    pub(super) known: HashSet<(String, String)>,
}

impl Catalog {
    pub(super) fn load(state: &AppState) -> AppResult<Self> {
        let folders = state.db.read(|conn| Ok(repo::folders::list(conn)?))?;
        let tag_list = state.db.read(|conn| Ok(tags::list(conn)?))?;
        let known = state
            .db
            .read(|conn| Ok(notes::import_sources(conn)?))?
            .into_iter()
            .collect();
        Ok(Self {
            known,
            folders: folders
                .into_iter()
                .map(|entry| {
                    (
                        (entry.folder.parent_id, entry.folder.name.to_lowercase()),
                        entry.folder.id,
                    )
                })
                .collect(),
            tags: tag_list
                .into_iter()
                .map(|entry| (entry.tag.name.to_lowercase(), entry.tag.id))
                .collect(),
            new_folders: 0,
            new_tags: 0,
        })
    }

    /// The folder at the end of `path`, below the top level; `None` for an empty path. Levels
    /// past the fifth are folded into the fifth.
    pub(super) fn folder(
        &mut self,
        state: &AppState,
        path: &[String],
    ) -> AppResult<Option<String>> {
        let mut parent: Option<String> = None;
        for raw in path.iter().take(MAX_FOLDER_DEPTH) {
            let name: String = raw.trim().chars().take(100).collect();
            if validation::validate_folder_name(&name).is_err() {
                continue;
            }
            let key = (parent.clone(), name.to_lowercase());
            let id = match self.folders.get(&key) {
                Some(id) => id.clone(),
                None => {
                    let created = folder_commands::upsert(
                        state,
                        FolderUpsertInput {
                            id: None,
                            name: name.clone(),
                            parent_id: parent.clone(),
                        },
                    )?;
                    self.new_folders += 1;
                    self.folders.insert(key, created.id.clone());
                    created.id
                }
            };
            parent = Some(id);
        }
        Ok(parent)
    }

    pub(super) fn tag(&mut self, state: &AppState, raw: &str) -> Option<String> {
        let name = validation::validate_tag_name(raw).ok()?.to_string();
        if let Some(id) = self.tags.get(&name.to_lowercase()) {
            return Some(id.clone());
        }
        let color = TAG_COLORS[self.new_tags % TAG_COLORS.len()];
        let created = tag_commands::upsert(
            state,
            TagUpsertInput {
                id: None,
                name: name.clone(),
                color_hex: color.to_string(),
                parent_id: None,
            },
        )
        .ok()?;
        self.new_tags += 1;
        self.tags.insert(name.to_lowercase(), created.id.clone());
        Some(created.id)
    }
}

pub(super) struct Incoming {
    pub(super) title: String,
    pub(super) document: Value,
    pub(super) tags: Vec<String>,
    pub(super) created: Option<String>,
    pub(super) updated: Option<String>,
    /// Where it came from and a hash of what it said (F17-06).
    pub(super) source: Option<(String, String)>,
}

pub(super) fn store_note(
    state: &AppState,
    catalog: &mut Catalog,
    folder_id: Option<&str>,
    incoming: Incoming,
) -> AppResult<()> {
    let title: String = incoming
        .title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(validation::MAX_TITLE_CHARS)
        .collect();
    let title = if title.is_empty() {
        "Sin título".to_string()
    } else {
        title
    };
    let json = serde_json::to_string(&incoming.document)
        .map_err(|e| AppError::Internal(format!("documento no válido: {e}")))?;
    let (document, attachment_ids) = derive(json)?;
    let source = incoming.source;
    let tag_ids: Vec<String> = incoming
        .tags
        .iter()
        .filter_map(|name| catalog.tag(state, name))
        .collect();
    let now = state.clock.now_string();
    let created = incoming.created.unwrap_or_else(|| now.clone());
    let updated = incoming.updated.unwrap_or_else(|| created.clone());
    let new = NewNote {
        id: state.ids.new_id(),
        folder_id: folder_id.map(str::to_string),
        title,
        content_json: Some(document.json),
        content_text: document.text,
        task_ids: Vec::new(),
        note_ids: Vec::new(),
        tag_ids,
        now: created.clone(),
    };
    state.db.write(|tx| {
        let note = notes::create(tx, &new)?;
        attachment_repo::sync_note(tx, &note.id, &attachment_ids)?;
        if updated != created {
            notes::set_updated_at(tx, &note.id, &updated)?;
        }
        if let Some((origin, hash)) = &source {
            notes::set_import_source(tx, &note.id, origin, hash)?;
        }
        Ok(())
    })?;
    if let Some(source) = source {
        catalog.known.insert(source);
    }
    Ok(())
}

pub(super) fn content_hash(parts: &[&str]) -> String {
    attachments::hash_of(parts.join("|").as_bytes())
}

pub(super) fn stored_image(state: &AppState, dir: &Path, bytes: &[u8]) -> Option<ImageFile> {
    attachments::add(state, dir, bytes)
        .ok()
        .map(|attachment| ImageFile {
            attachment_id: attachment.id,
            mime: attachment.mime,
        })
}
