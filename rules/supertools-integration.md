# Supertools Integration Rules

## PFM (Plannotator Flavored Markdown)

When generating plans (inside plan mode), use these Plannotator Flavored Markdown extensions to produce richer, more reviewable plans:

- **Callouts**: Use `> [!NOTE]`, `> [!TIP]`, `> [!IMPORTANT]`, `> [!WARNING]`, `> [!CAUTION]` for highlighted blocks
- **Code-file links**: Reference source files with `[filename](file:///absolute/path/to/file)` or specific lines with `[fn](file:///path#L10-L20)`
- **Task lists**: Use `- [ ]` for pending items and `- [x]` for completed items
- **Hex swatches**: Inline color codes like `#ff6b6b` render as colored badges
- **Mermaid diagrams**: Use fenced ` ```mermaid ` blocks for flowcharts, sequence diagrams, etc.
- **Tables**: Use standard markdown tables for structured data comparisons
- **Wiki-links**: Use `[[page-name]]` for cross-referencing within plans

## Feedback Protocol

When receiving feedback from Supertools (after plan denial or code review):

1. Parse the structured feedback including line-referenced annotations
2. Address each annotation specifically, referencing the original line numbers
3. If annotations request code changes, implement them directly
4. If annotations ask questions, answer them in the revised plan
5. Do not summarize or paraphrase annotations — act on them

## Execution Style

- Always execute Supertools commands directly using the shell — never ask the user to copy-paste commands
- Set `SUPERTOOLS_ORIGIN=antigravity-cli` when invoking the CLI to ensure proper origin detection
- Wait patiently for browser-based review sessions to complete (they may take minutes)
- Do not kill, restart, or duplicate Supertools server sessions while a review is in progress
