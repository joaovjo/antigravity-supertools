# Antigravity Supertools

Cooperative multi-agent engineering suite for [Antigravity](https://antigravity.google). Consolidates the best of [Plannotator](https://github.com/backnotprop/plannotator) (visual plan & code review) and [Matt Pocock's Skills](https://github.com/mattpocock/skills) (engineering workflows) into a single, lightweight, configuration-only plugin.

No binary dependencies. No hooks. No scripts. One slash command, four specialized agents, TOON-serialized output.

## Install

```bash
agy plugin install ./antigravity-supertools
```

Verify:

```bash
agy plugin list
```

## Usage

Every interaction goes through a single entry point:

```
/supertools <your request>
```

The `/supertools` skill classifies your intent and delegates to the right agent:

| Intent | Agent | Examples |
|--------|-------|---------|
| Plan, design, grill, domain modeling | **Architect** | "plan a new auth module", "grill me on this design", "update the glossary" |
| Implement, TDD, prototype | **Developer** | "implement with TDD", "build a prototype", "write the feature" |
| Code review, diff analysis, annotations | **Reviewer** | "review my changes", "annotate this file", "check the last diff" |
| Bug diagnosis, regression tracing | **Debugger** | "there's a bug in auth", "diagnose this failure", "why is this slow" |
| Ultra-terse communication | (caveman mode) | "caveman mode", "be terse" |
| Session handoff | (handoff) | "handoff", "summarize for next session" |

## Architecture

### Four Agents

| Agent | Consolidates | Core Loop |
|-------|-------------|-----------|
| **Architect** | grill-me, grill-with-docs, domain-modeling, to-prd, to-issues, improve-codebase-architecture | Grill → plan → update memory |
| **Developer** | tdd, prototype, implement, codebase-design | RED → GREEN → REFACTOR → self-reflect |
| **Reviewer** | plannotator-review, plannotator-annotate, plannotator-last, plannotator-visual-explainer | Diff → annotate → TOON table output |
| **Debugger** | diagnosing-bugs | Reproduce → minimise → hypothesise → instrument → fix |

### LLM-Wiki Memory

The Architect maintains a `memory/` folder of interlinked markdown using `[[wikilinks]]`:

```
memory/
├── domain.md          # Glossary, ubiquitous language
├── architecture.md    # Module map, seams, design decisions
└── index.md           # Unified TOON file index
```

### TOON Format

All structured data uses [TOON](https://toonformat.dev) (Token-Oriented Object Notation) for 30–60% token savings:

```toon
[3]{task,status,assignee}:
Implement auth middleware,done,developer
Write regression test,in-progress,debugger
Update domain glossary,todo,architect
```

## File Structure

```
antigravity-supertools/
├── plugin.json
├── README.md
├── rules/
│   ├── team-orchestration.md
│   └── caveman-mode.md
├── agents/
│   ├── architect.md
│   ├── developer.md
│   ├── reviewer.md
│   └── debugger.md
└── skills/
    └── supertools/
        └── SKILL.md
```

## License

MIT OR Apache-2.0
