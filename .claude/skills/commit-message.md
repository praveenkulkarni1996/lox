# Commit Message

Use this skill when writing a git commit message for this project.

## Rules

- **Source from the diff**: Run `git diff --cached` and describe what changed. Do not summarize the conversation.
- **Subject line**: Short imperative phrase ending with a period, ≤72 characters.
- **Body paragraphs**: Describe *what* changed and *why*. Use `*` bullets for API listings.
- **No AI footers**: Never include `Co-Authored-By` or any attribution to Claude/Anthropic.
- **No trailing summaries**: The diff is the record; don't append a recap of what was done.

## Tips & Tricks

- Name Rust types with the `{}` suffix (e.g. `Environment{}`) to distinguish types from variables or modules.
