# Commit Message & Doc Comments

Use this skill when writing a git commit message or Rust doc comments for this project.

## Commit Message Rules

- **Source from the diff**: Run `git diff --cached` and describe what changed. Do not summarize the conversation.
- **Subject line**: Short imperative phrase ending with a period, ≤72 characters.
- **Body paragraphs**: Describe *what* changed and *why*. Use `*` bullets for API listings.
- **No AI footers**: Never include `Co-Authored-By` or any attribution to Claude/Anthropic.
- **No trailing summaries**: The diff is the record; don't append a recap of what was done.

## Doc Comment Rules

- **Module-level (`//!`)**: Every `lib.rs` should open with a `//!` block describing the crate's purpose and linking the relevant Crafting Interpreters chapter.
- **Public items (`///`)**: Every public type, variant, function, and method gets a `///` doc comment.
- **First line**: A short imperative or noun-phrase summary — one sentence, no trailing blank line before the body.
- **Reference links**: When the code implements a section from Crafting Interpreters, include a `Reference: <URL>` line at the end of the doc comment.
- **`# Examples`**: Public API entry points (like `run()`) should include a runnable `# Examples` section with `assert_eq!`.
- **Internal functions**: Private `fn` helpers get a `///` comment when their purpose isn't obvious from the name — especially eval functions that implement specific Lox semantics (short-circuit, truthiness, etc.).

## Tips & Tricks

- Name Rust types with the `{}` suffix (e.g. `Environment{}`) to distinguish types from variables or modules.
