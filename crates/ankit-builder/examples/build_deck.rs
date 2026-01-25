//! Example: Build an Anki deck from TOML
//!
//! This example demonstrates how to use ankit-builder to:
//! 1. Parse a TOML deck definition
//! 2. Generate an .apkg file
//!
//! Run with: cargo run -p ankit-builder --example build_deck

use ankit_builder::DeckBuilder;

const EXAMPLE_TOML: &str = r#"
[package]
name = "Spanish Basics"
version = "1.0.0"
author = "Example Author"

[[models]]
name = "Basic Spanish"
fields = ["Spanish", "English", "Example"]

[[models.templates]]
name = "Spanish -> English"
front = "{{Spanish}}"
back = """
{{FrontSide}}
<hr>
<b>{{English}}</b>
<br><br>
<i>{{Example}}</i>
"""

[[models.templates]]
name = "English -> Spanish"
front = "{{English}}"
back = """
{{FrontSide}}
<hr>
<b>{{Spanish}}</b>
"""

[[decks]]
name = "Spanish::Basics"
description = "Basic Spanish vocabulary for beginners"

[[notes]]
deck = "Spanish::Basics"
model = "Basic Spanish"
tags = ["greetings", "basics"]

[notes.fields]
Spanish = "Hola"
English = "Hello"
Example = "Hola, como estas?"

[[notes]]
deck = "Spanish::Basics"
model = "Basic Spanish"
tags = ["greetings", "basics"]

[notes.fields]
Spanish = "Adios"
English = "Goodbye"
Example = "Adios, hasta manana!"

[[notes]]
deck = "Spanish::Basics"
model = "Basic Spanish"
tags = ["numbers", "basics"]

[notes.fields]
Spanish = "uno"
English = "one"
Example = "Tengo uno."

[[notes]]
deck = "Spanish::Basics"
model = "Basic Spanish"
tags = ["numbers", "basics"]

[notes.fields]
Spanish = "dos"
English = "two"
Example = "Son las dos."

[[notes]]
deck = "Spanish::Basics"
model = "Basic Spanish"
tags = ["numbers", "basics"]

[notes.fields]
Spanish = "tres"
English = "three"
Example = "Tres amigos."
"#;

fn main() -> ankit_builder::Result<()> {
    println!("Parsing TOML deck definition...");

    let builder = DeckBuilder::parse(EXAMPLE_TOML)?;
    let definition = builder.definition();

    println!("  Package: {}", definition.package.name);
    println!("  Models: {}", definition.models.len());
    println!("  Decks: {}", definition.decks.len());
    println!("  Notes: {}", definition.notes.len());

    // Generate .apkg file
    let output_path = std::env::temp_dir().join("spanish_basics.apkg");
    println!("\nGenerating .apkg file: {}", output_path.display());

    builder.write_apkg(&output_path)?;

    let metadata = std::fs::metadata(&output_path)?;
    println!("  Created: {} bytes", metadata.len());

    println!("\nDone! You can import this file into Anki.");
    println!("File location: {}", output_path.display());

    Ok(())
}
