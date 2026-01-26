//! .apkg file generation.
//!
//! Creates Anki package files that can be imported directly into Anki.

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use tempfile::TempDir;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::error::Result;
use crate::schema::DeckDefinition;
use crate::sql::{DEFAULT_CONF, DEFAULT_DCONF, FIELD_SEPARATOR, SCHEMA};

/// Builder for creating .apkg files from deck definitions.
pub struct ApkgBuilder {
    definition: DeckDefinition,
    media_base_path: Option<std::path::PathBuf>,
}

impl ApkgBuilder {
    /// Create a new builder from a deck definition.
    pub fn new(definition: DeckDefinition) -> Self {
        Self {
            definition,
            media_base_path: None,
        }
    }

    /// Set the base path for resolving media file paths.
    pub fn media_base_path(mut self, path: impl AsRef<Path>) -> Self {
        self.media_base_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Build the .apkg file and write it to the specified path.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("collection.anki2");

        // Create and populate the SQLite database
        let conn = Connection::open(&db_path)?;
        self.create_database(&conn)?;

        // Create the ZIP file
        let file = std::fs::File::create(path)?;
        let mut zip = ZipWriter::new(file);

        // Add the database file
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("collection.anki2", options)?;
        let db_bytes = std::fs::read(&db_path)?;
        zip.write_all(&db_bytes)?;

        // Add media manifest and files
        let media_manifest = self.build_media_manifest()?;
        zip.start_file("media", options)?;
        zip.write_all(media_manifest.as_bytes())?;

        // Add media files with numeric names
        for (index, media) in self.definition.media.iter().enumerate() {
            let source_path = self.resolve_media_path(&media.path)?;
            let content = std::fs::read(&source_path)?;
            zip.start_file(index.to_string(), options)?;
            zip.write_all(&content)?;
        }

        zip.finish()?;
        Ok(())
    }

    /// Create the SQLite database with all content.
    fn create_database(&self, conn: &Connection) -> Result<()> {
        // Create schema
        conn.execute_batch(SCHEMA)?;

        // Generate timestamps and IDs
        let now = current_timestamp();
        let now_ms = now * 1000;

        // Build model and deck JSON
        let models_json = self.build_models_json(now);
        let decks_json = self.build_decks_json(now);

        // Insert collection row
        conn.execute(
            "INSERT INTO col (id, crt, mod, scm, ver, dty, usn, ls, conf, models, decks, dconf, tags)
             VALUES (1, ?, ?, ?, 11, 0, -1, 0, ?, ?, ?, ?, '{}')",
            rusqlite::params![now, now_ms, now_ms, DEFAULT_CONF, models_json, decks_json, DEFAULT_DCONF],
        )?;

        // Insert notes and cards
        let mut note_id_gen = now_ms;
        let mut card_id_gen = now_ms;

        for note_def in &self.definition.notes {
            let model = self.definition.get_model(&note_def.model).unwrap();
            let deck = self.definition.get_deck(&note_def.deck).unwrap();
            let deck_id = deck.id.unwrap_or_else(|| generate_id(&deck.name));
            let model_id = model.id.unwrap_or_else(|| generate_id(&model.name));

            // Insert note
            let note_id = note_id_gen;
            note_id_gen += 1;

            let guid = note_def
                .guid
                .clone()
                .unwrap_or_else(|| generate_guid(note_id));

            // Convert markdown fields to HTML before storing
            let html_fields = note_def.fields_as_html(&model.markdown_fields);
            let fields_str = model
                .fields
                .iter()
                .map(|f| html_fields.get(f).cloned().unwrap_or_default())
                .collect::<Vec<_>>()
                .join(&FIELD_SEPARATOR.to_string());
            let sort_field = note_def
                .fields_ordered(model)
                .get(model.sort_field_index())
                .cloned()
                .unwrap_or_default();
            let checksum = compute_checksum(&sort_field);

            conn.execute(
                "INSERT INTO notes (id, guid, mid, mod, usn, tags, flds, sfld, csum, flags, data)
                 VALUES (?, ?, ?, ?, -1, ?, ?, ?, ?, 0, '')",
                rusqlite::params![
                    note_id,
                    guid,
                    model_id,
                    now,
                    note_def.tags_string(),
                    fields_str,
                    sort_field,
                    checksum
                ],
            )?;

            // Insert cards (one per template)
            for (ord, _template) in model.templates.iter().enumerate() {
                let card_id = card_id_gen;
                card_id_gen += 1;

                conn.execute(
                    "INSERT INTO cards (id, nid, did, ord, mod, usn, type, queue, due, ivl, factor, reps, lapses, left, odue, odid, flags, data)
                     VALUES (?, ?, ?, ?, ?, -1, 0, 0, ?, 0, 0, 0, 0, 0, 0, 0, 0, '')",
                    rusqlite::params![card_id, note_id, deck_id, ord, now, card_id_gen],
                )?;
            }
        }

        Ok(())
    }

    /// Build the models JSON for the col table.
    fn build_models_json(&self, now: i64) -> String {
        let mut models: HashMap<String, serde_json::Value> = HashMap::new();

        for model in &self.definition.models {
            let model_id = model.id.unwrap_or_else(|| generate_id(&model.name));

            let fields: Vec<serde_json::Value> = model
                .fields
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    serde_json::json!({
                        "name": name,
                        "ord": i,
                        "sticky": false,
                        "rtl": false,
                        "font": "Arial",
                        "size": 20,
                        "media": []
                    })
                })
                .collect();

            let templates: Vec<serde_json::Value> = model
                .templates
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    serde_json::json!({
                        "name": t.name,
                        "ord": i,
                        "qfmt": t.front,
                        "afmt": t.back,
                        "bqfmt": "",
                        "bafmt": "",
                        "did": null,
                        "bfont": "",
                        "bsize": 0
                    })
                })
                .collect();

            let model_obj = serde_json::json!({
                "id": model_id,
                "name": model.name,
                "type": 0,
                "mod": now,
                "usn": -1,
                "sortf": model.sort_field_index(),
                "did": null,
                "tmpls": templates,
                "flds": fields,
                "css": model.css.clone().unwrap_or_else(default_css),
                "latexPre": "\\documentclass[12pt]{article}\n\\special{papersize=3in,5in}\n\\usepackage{amssymb,amsmath}\n\\pagestyle{empty}\n\\setlength{\\parindent}{0in}\n\\begin{document}\n",
                "latexPost": "\\end{document}",
                "latexsvg": false,
                "req": build_requirements(&model.templates, &model.fields)
            });

            models.insert(model_id.to_string(), model_obj);
        }

        serde_json::to_string(&models).unwrap()
    }

    /// Build the decks JSON for the col table.
    fn build_decks_json(&self, now: i64) -> String {
        let mut decks: HashMap<String, serde_json::Value> = HashMap::new();

        // Always include the default deck
        decks.insert(
            "1".to_string(),
            serde_json::json!({
                "id": 1,
                "mod": now,
                "name": "Default",
                "usn": -1,
                "lrnToday": [0, 0],
                "revToday": [0, 0],
                "newToday": [0, 0],
                "timeToday": [0, 0],
                "collapsed": false,
                "browserCollapsed": false,
                "desc": "",
                "dyn": 0,
                "conf": 1,
                "extendNew": 10,
                "extendRev": 50
            }),
        );

        for deck in &self.definition.decks {
            let deck_id = deck.id.unwrap_or_else(|| generate_id(&deck.name));
            let deck_obj = serde_json::json!({
                "id": deck_id,
                "mod": now,
                "name": deck.name,
                "usn": -1,
                "lrnToday": [0, 0],
                "revToday": [0, 0],
                "newToday": [0, 0],
                "timeToday": [0, 0],
                "collapsed": false,
                "browserCollapsed": false,
                "desc": deck.description.clone().unwrap_or_default(),
                "dyn": 0,
                "conf": 1,
                "extendNew": 10,
                "extendRev": 50
            });

            decks.insert(deck_id.to_string(), deck_obj);
        }

        serde_json::to_string(&decks).unwrap()
    }

    /// Build the media manifest JSON.
    fn build_media_manifest(&self) -> Result<String> {
        let manifest: HashMap<String, &str> = self
            .definition
            .media
            .iter()
            .enumerate()
            .map(|(i, m)| (i.to_string(), m.name.as_str()))
            .collect();

        Ok(serde_json::to_string(&manifest).unwrap())
    }

    /// Resolve a media file path.
    fn resolve_media_path(&self, path: &str) -> Result<std::path::PathBuf> {
        let path = Path::new(path);
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else if let Some(ref base) = self.media_base_path {
            Ok(base.join(path))
        } else {
            Ok(path.to_path_buf())
        }
    }
}

/// Get current Unix timestamp in seconds.
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Generate a stable ID from a string (for models and decks).
fn generate_id(name: &str) -> i64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    // Ensure positive and reasonable size
    (hasher.finish() & 0x7FFF_FFFF_FFFF) as i64
}

/// Generate a GUID for a note.
fn generate_guid(note_id: i64) -> String {
    // Base91 encoding similar to Anki
    const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+,-./:;<=>?@[]^_`{|}~";
    let mut n = note_id as u64;
    let mut result = String::new();
    while n > 0 {
        result.push(CHARS[(n % 91) as usize] as char);
        n /= 91;
    }
    result
}

/// Compute a checksum for the sort field.
fn compute_checksum(sort_field: &str) -> i64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Strip HTML and compute hash
    let text = strip_html(sort_field);
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    (hasher.finish() & 0xFFFFFFFF) as i64
}

/// Simple HTML stripping.
fn strip_html(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
}

/// Build model requirements (which fields must be non-empty for each template).
fn build_requirements(
    templates: &[crate::schema::TemplateDef],
    fields: &[String],
) -> Vec<serde_json::Value> {
    templates
        .iter()
        .enumerate()
        .map(|(ord, template)| {
            // Find fields referenced in the front template
            let referenced: Vec<usize> = fields
                .iter()
                .enumerate()
                .filter(|(_, f)| template.front.contains(&format!("{{{{{}}}}}", f)))
                .map(|(i, _)| i)
                .collect();

            if referenced.is_empty() {
                // If no fields detected, require first field
                serde_json::json!([ord, "any", [0]])
            } else {
                serde_json::json!([ord, "any", referenced])
            }
        })
        .collect()
}

/// Default CSS for cards.
fn default_css() -> String {
    r#".card {
    font-family: arial;
    font-size: 20px;
    text-align: center;
    color: black;
    background-color: white;
}"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::DeckDefinition;
    use tempfile::tempdir;

    #[test]
    fn test_generate_guid() {
        let guid = generate_guid(1234567890);
        assert!(!guid.is_empty());
        // Should be deterministic
        assert_eq!(guid, generate_guid(1234567890));
    }

    #[test]
    fn test_generate_id() {
        let id = generate_id("Test Model");
        assert!(id > 0);
        // Should be deterministic
        assert_eq!(id, generate_id("Test Model"));
        // Different names should give different IDs
        assert_ne!(id, generate_id("Other Model"));
    }

    #[test]
    fn test_strip_html() {
        assert_eq!(strip_html("<b>Hello</b> World"), "Hello World");
        assert_eq!(strip_html("No HTML"), "No HTML");
        assert_eq!(strip_html("<div><p>Nested</p></div>"), "Nested");
    }

    #[test]
    fn test_write_apkg() {
        let toml = r#"
[package]
name = "Test"

[[models]]
name = "Basic"
fields = ["Front", "Back"]

[[models.templates]]
name = "Card 1"
front = "{{Front}}"
back = "{{Back}}"

[[decks]]
name = "Test Deck"

[[notes]]
deck = "Test Deck"
model = "Basic"

[notes.fields]
Front = "Question"
Back = "Answer"
"#;

        let def = DeckDefinition::parse(toml).unwrap();
        let builder = ApkgBuilder::new(def);

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.apkg");

        builder.write_to_file(&path).unwrap();

        // Verify the file exists and is a valid ZIP
        assert!(path.exists());
        let file = std::fs::File::open(&path).unwrap();
        let archive = zip::ZipArchive::new(file).unwrap();

        // Check expected files
        let file_names: Vec<_> = archive.file_names().collect();
        assert!(file_names.contains(&"collection.anki2"));
        assert!(file_names.contains(&"media"));
    }
}
