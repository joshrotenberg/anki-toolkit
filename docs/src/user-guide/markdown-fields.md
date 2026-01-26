# Markdown Fields

Write note content in Markdown for cleaner, more readable TOML files.

## Enabling Markdown

Specify which fields use Markdown in your model definition:

```toml
[[models]]
name = "Vocabulary"
fields = ["Word", "Definition", "Examples"]
markdown_fields = ["Definition", "Examples"]
```

## Supported Syntax

### Text Formatting

| Markdown | HTML Result |
|----------|-------------|
| `**bold**` | **bold** |
| `*italic*` | *italic* |
| `~~strikethrough~~` | ~~strikethrough~~ |
| `` `code` `` | `code` |

### Lists

```markdown
- Item 1
- Item 2
  - Nested item

1. First
2. Second
3. Third
```

### Links

```markdown
[Link text](https://example.com)
```

### Code Blocks

````markdown
```python
def hello():
    print("Hello!")
```
````

### Blockquotes

```markdown
> This is a quote
```

## Example Note

```toml
[[notes]]
deck = "Programming"
model = "Concept"

[notes.fields]
Term = "List Comprehension"
Definition = """
A **concise** way to create lists in Python.

Syntax: `[expr for item in iterable]`
"""
Example = """
```python
squares = [x**2 for x in range(10)]
```
"""
```

## How It Works

- **Push to Anki**: Markdown is converted to HTML
- **Pull from Anki**: HTML is converted back to Markdown
- Non-markdown fields are left unchanged

## Best Practices

1. **Use markdown_fields for content-heavy fields** like definitions, examples, explanations
2. **Keep simple fields as plain text** like single words, dates, IDs
3. **Test the conversion** by doing a round-trip: push, then pull and compare
