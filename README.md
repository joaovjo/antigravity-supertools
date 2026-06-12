# Antigravity Supertools

Unified engineering toolkit for [Antigravity CLI](https://antigravity.google) — interactive plan review, code review, annotations, TDD, diagnostics, grilling sessions, and 20+ productivity skills.

Combines the best of [Plannotator](https://github.com/backnotprop/plannotator) (visual plan & code review) with [Matt Pocock's Skills](https://github.com/mattpocock/skills) (engineering workflows) into a single, self-contained plugin powered by a custom Rust CLI binary.

## Install

### Via Marketplace

```bash
agy plugin install antigravity-supertools
```

### From Source

```bash
agy plugin install ./antigravity-supertools
```

### Prerequisites

The `supertools` binary must be on your `PATH` for plan mode hooks and review commands to work.

**Build from source:**

```bash
cd supertools/
cargo build --release
# Copy target/release/supertools to a directory in your PATH
```

**Or download a pre-built binary** from [GitHub Releases](https://github.com/joaovjo/antigravity-supertools/releases).

## Skills Reference

### Plan Review & Annotations (from Plannotator)

| Skill | Description |
|-------|-------------|
| `/plannotator-annotate` | Open annotation UI for a markdown file, HTML, URL, or folder |
| `/plannotator-compound` | Analyze denied plan patterns and produce a feedback dashboard |
| `/plannotator-last` | Annotate the agent's last response and revise based on feedback |
| `/plannotator-review` | Open interactive code review for current changes or a PR URL |
| `/plannotator-visual-explainer` | Generate visual HTML plans, diagrams, and explainers |

### Engineering (from Matt Pocock Skills)

| Skill | Description |
|-------|-------------|
| `/diagnose` | Disciplined diagnosis loop for hard bugs: reproduce → minimise → hypothesise → instrument → fix |
| `/grill-with-docs` | Grilling session that challenges your plan against the domain model, updates CONTEXT.md and ADRs |
| `/triage` | Triage issues through a state machine of triage roles |
| `/improve-codebase-architecture` | Find deepening opportunities informed by domain language and ADRs |
| `/setup-supertools` | Scaffold per-repo config (issue tracker, triage labels, docs layout) |
| `/tdd` | Test-driven development with red-green-refactor loop |
| `/to-issues` | Break any plan, spec, or PRD into independently-grabbable issues |
| `/to-prd` | Synthesize current conversation context into a PRD |
| `/zoom-out` | Get broader context or higher-level perspective on code |
| `/prototype` | Build a throwaway prototype to flesh out a design |

### Productivity

| Skill | Description |
|-------|-------------|
| `/caveman` | Ultra-compressed communication mode (~75% token savings) |
| `/grill-me` | Get relentlessly interviewed about a plan until every decision is resolved |
| `/handoff` | Compact conversation into a handoff document for another agent |
| `/teach` | Learn a new skill or concept over multiple sessions |
| `/write-a-skill` | Create new skills with proper structure and progressive disclosure |

## Plan Mode Integration

This plugin automatically integrates with Antigravity CLI's plan mode:

1. **Entering plan mode** → `supertools improve-context` injects PFM (Plannotator Flavored Markdown) context
2. **Exiting plan mode** → `supertools` opens a browser-based review UI where you can approve, deny, or annotate the plan

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SUPERTOOLS_REMOTE` | Set to `1` for remote/SSH sessions |
| `SUPERTOOLS_PORT` | Fixed port (default: random locally) |
| `SUPERTOOLS_BROWSER` | Custom browser to open |

## License

MIT OR Apache-2.0
