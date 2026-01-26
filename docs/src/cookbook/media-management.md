# Media Management

Patterns for managing audio, images, and other media in Anki.

## Audit Media Files

Find orphaned files and missing references:

```rust,ignore
use ankit_engine::Engine;

let engine = Engine::new();

let report = engine.media().audit().await?;

println!("Orphaned files (in folder but not used):");
for file in &report.orphaned {
    println!("  {}", file);
}

println!("\nMissing files (referenced but not found):");
for file in &report.missing {
    println!("  {}", file);
}
```

## Clean Up Orphaned Media

Preview what would be deleted:

```rust,ignore
let preview = engine.media()
    .cleanup(true)  // dry_run = true
    .await?;

println!("Would delete {} files:", preview.deleted.len());
for file in &preview.deleted {
    println!("  {}", file);
}
```

Actually delete orphaned files:

```rust,ignore
let result = engine.media()
    .cleanup(false)  // dry_run = false
    .await?;

println!("Deleted {} orphaned files", result.deleted.len());
```

## Store Media Files

Store a file from various sources:

```rust,ignore
use ankit::StoreMediaParams;

// From base64
let params = StoreMediaParams::from_base64(
    "test.txt",
    "SGVsbG8gV29ybGQ=",  // "Hello World"
);
let filename = engine.client().media().store(params).await?;

// From URL
let params = StoreMediaParams::from_url(
    "image.png",
    "https://example.com/image.png",
);
let filename = engine.client().media().store(params).await?;

// From local file
let params = StoreMediaParams::from_path(
    "audio.mp3",
    "/path/to/audio.mp3",
);
let filename = engine.client().media().store(params).await?;
```

## List Media Files

```rust,ignore
// List all media files
let all_files = engine.client().media().list("*").await?;

// List by extension
let mp3_files = engine.client().media().list("*.mp3").await?;
let images = engine.client().media().list("*.{png,jpg,gif}").await?;

// List by prefix
let prefixed = engine.client().media().list("japanese_*").await?;
```

## Get Media Directory Path

```rust,ignore
let media_dir = engine.client().media().directory().await?;
println!("Media stored at: {}", media_dir);
```

## Retrieve Media Content

```rust,ignore
// Get file content as base64
let base64_data = engine.client().media()
    .retrieve("audio.mp3")
    .await?;

// Decode if needed
use base64::{Engine as _, engine::general_purpose};
let bytes = general_purpose::STANDARD.decode(&base64_data)?;
```

## Delete Media Files

```rust,ignore
engine.client().media()
    .delete("old_audio.mp3")
    .await?;
```

## Add Notes with Media

### Audio

```rust,ignore
use ankit::{NoteBuilder, MediaAttachment};

let note = NoteBuilder::new("Vocabulary", "Basic")
    .field("Front", "hello")
    .field("Back", "world")
    .audio(MediaAttachment {
        url: Some("https://example.com/hello.mp3".to_string()),
        filename: "hello.mp3".to_string(),
        fields: vec!["Front".to_string()],  // Insert into Front field
        ..Default::default()
    })
    .build();

engine.client().notes().add(note).await?;
```

### Images

```rust,ignore
let note = NoteBuilder::new("Geography", "Basic")
    .field("Front", "What country is this?")
    .field("Back", "France")
    .picture(MediaAttachment {
        path: Some("/path/to/france_map.png".to_string()),
        filename: "france_map.png".to_string(),
        fields: vec!["Front".to_string()],
        ..Default::default()
    })
    .build();
```

### Video

```rust,ignore
let note = NoteBuilder::new("Language", "Basic")
    .field("Front", "Watch the gesture")
    .field("Back", "Hello")
    .video(MediaAttachment {
        url: Some("https://example.com/gesture.mp4".to_string()),
        filename: "gesture.mp4".to_string(),
        fields: vec!["Front".to_string()],
        ..Default::default()
    })
    .build();
```

## Bulk Media Check

Find notes with missing media:

```rust,ignore
let note_ids = engine.client().notes()
    .find("deck:Vocabulary")
    .await?;

let notes = engine.client().notes().info(&note_ids).await?;
let media_files: std::collections::HashSet<_> =
    engine.client().media().list("*").await?.into_iter().collect();

for note in notes {
    for (_, field) in &note.fields {
        // Check for [sound:...] references
        if field.value.contains("[sound:") {
            // Extract filename and check if it exists
            // ...
        }
        // Check for <img src="..."> references
        if field.value.contains("<img") {
            // ...
        }
    }
}
```
