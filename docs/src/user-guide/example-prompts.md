# Example Prompts

Here are example prompts you can use with Claude to manage your Anki decks.

## Creating Flashcards

> "Add a note to my Japanese deck with front 'hello' and back 'konnichiwa'"

> "Create a flashcard in Spanish::Vocabulary with the word 'gato' meaning 'cat'"

> "Add 5 basic vocabulary cards for common French greetings"

## Searching and Browsing

> "Show me all decks in my collection"

> "Find notes in my medical deck tagged with 'cardiology'"

> "How many cards are due today?"

> "List all cards I've reviewed in the last week"

## Study Analysis

> "Give me a study summary for my Japanese deck over the last 30 days"

> "What's my retention rate in the Spanish deck?"

> "Show me a health report for my vocabulary deck"

> "Find cards I'm struggling with (leeches) in any deck"

## Managing Cards

> "Suspend all cards tagged 'deprecated' in my physics deck"

> "Reset the progress on my 'Chapter 1' cards"

> "Unsuspend the cards I suspended yesterday"

> "Find cards with ease below 200% and tag them as 'needs-review'"

## Organization

> "Clone my 'Japanese N5' deck to 'Japanese N5 Backup'"

> "Merge 'Vocab Part 1' and 'Vocab Part 2' into a single 'Vocabulary' deck"

> "Move all notes tagged 'advanced' to the 'Advanced Spanish' deck"

## Deduplication

> "Find duplicate notes in my vocabulary deck"

> "Preview what would be deleted if I remove duplicates from my deck"

> "Remove duplicates in my imported deck, keeping the one with the most content"

## Tags

> "Add the tag 'reviewed-2024' to all notes in my history deck"

> "Remove the 'temp' tag from all notes"

> "Rename the tag 'oldname' to 'newname' across my entire collection"

## Import/Export

> "Export my Japanese deck to TOML format"

> "Import this TOML deck definition into Anki"

> "Compare my vocabulary.toml file against what's currently in Anki"

## Media Management

> "Check for orphaned media files in my collection"

> "Show me what media files would be deleted if I clean up"

## Tips for Better Results

### Be Specific About Decks
Always mention which deck you're working with:
- Good: "Add a card to my Spanish deck"
- Less good: "Add a card"

### Use Preview for Bulk Operations
Before making big changes:
- "Preview removing duplicates" before "Remove duplicates"
- "Show what would be suspended" before "Suspend cards"

### Combine Operations
Claude can chain multiple operations:
> "Find all cards with more than 5 lapses in my Japanese deck, tag them as 'struggling', and give me a summary"

### Ask for Explanations
Claude can explain what it's doing:
> "What tools would you use to clean up my deck?"
