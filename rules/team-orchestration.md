# Team Orchestration

Central coordination rules for the cooperative multi-agent system. All agents read this file.

## Agent Roles

| Agent | Owns | Boundary |
|-------|------|----------|
| **Architect** | Planning, grilling, domain modeling, memory maintenance, PRDs, issue slicing, architecture review | Never writes production code or tests |
| **Developer** | Implementation, TDD loops, prototyping, codebase design | Never plans scope or triages — takes tasks from Architect's checklist |
| **Reviewer** | Code review, diff annotations, visual explanations | Never modifies source code — only emits feedback |
| **Debugger** | Bug diagnosis, reproduction, root-cause analysis, regression tests | Only writes code to fix verified bugs — never implements new features |

## Delegation Protocol

When `/supertools` receives a user request, classify intent:

1. **Plan / design / grill / domain / architecture / PRD / issues** → Architect
2. **Implement / TDD / prototype / build / code** → Developer
3. **Review / diff / annotate / check / explain visually** → Reviewer
4. **Bug / debug / diagnose / regression / slow / broken / failing** → Debugger
5. **Caveman / terse / brief** → Toggle caveman-mode rule (no agent delegation)
6. **Handoff / summarize for next session** → Generate TOON handoff (no agent delegation)

If intent spans multiple agents, orchestrate sequentially: Architect plans → Developer implements → Reviewer validates. State each delegation explicitly.

## TOON Table Schemas

All structured inter-agent data uses [TOON](https://toonformat.dev) tables. These are the canonical schemas:

### Checklist

Used by all agents to track task progress.

```toon
[count]{task,status,assignee}:
Design auth module interface,done,architect
Implement token validation,in-progress,developer
Review auth diff,todo,reviewer
```

Status values: `todo`, `in-progress`, `done`, `blocked`.

### Review Annotations

Emitted by the Reviewer agent.

```toon
[count]{file,line,type,issue,fix}:
src/auth.ts,42,bug,Token expiry uses < instead of <=,Change to <=
src/auth.ts,78,design,Handler is shallow — wraps gateway 1:1,Absorb into gateway module
src/db.ts,15,perf,N+1 query in user lookup,Batch with IN clause
```

Type values: `bug`, `style`, `perf`, `security`, `design`, `question`.

### Diagnostic Log

Used by the Debugger agent to track hypothesis testing.

```toon
[count]{symptom,hypothesis,test,status}:
Auth returns 401 on valid token,Token expiry off-by-one,Unit test with edge timestamp,confirmed
Auth returns 401 on valid token,Clock skew between services,Compare server times,refuted
```

Status values: `untested`, `testing`, `confirmed`, `refuted`.

### File Index

Maintained by the Architect in `memory/index.md`.

```toon
[count]{file,purpose,exports,dependencies}:
src/auth.ts,Token validation and session management,validateToken;createSession,src/db.ts;src/config.ts
src/db.ts,Database connection and query layer,query;transaction,pg
src/config.ts,Environment configuration loader,getConfig,dotenv
```

## Handoff Protocol

When user requests a handoff, generate a compact TOON-based summary:

```markdown
# Handoff

## Decisions Made
[count]{decision,rationale,adr}:
Use JWT for auth,Stateless and horizontally scalable,docs/adr/0001-jwt-auth.md
Postgres for write model,ACID guarantees needed,docs/adr/0002-postgres.md

## Current State
[count]{task,status,assignee}:
...current checklist...

## Open Questions
[count]{question,context,blocker}:
Should sessions expire?,Product hasn't decided TTL,blocks auth implementation

## Artifacts
[count]{type,path}:
PRD,docs/prd-auth.md
ADR,docs/adr/0001-jwt-auth.md

## Suggested Next Steps
1. Resolve session TTL with product
2. Developer: implement token validation via TDD
3. Reviewer: review auth module diff
```

Do not duplicate content already in artifacts — reference by path. Redact sensitive information.

## Memory Protocol

The Architect maintains `memory/` using these rules:

1. **Create lazily** — only when there is content to write
2. **Use `[[wikilinks]]`** — every domain term, module name, or decision reference should cross-link
3. **TOON for data** — file indexes, component catalogs, and relationship maps use TOON tables
4. **Prose for narrative** — architecture rationale, domain explanations, and design context use plain markdown
5. **Never store code** — reference files by path, don't inline source code
6. **Update inline** — capture terms and decisions the moment they crystallize, don't batch

## Feedback Protocol

When receiving structured feedback (annotations, review results, diagnostic findings):

1. Parse the TOON table directly — do not summarize or paraphrase
2. Address each row specifically, referencing file and line
3. If a row requests a code change, implement it directly
4. If a row asks a question, answer it before proceeding
5. Update the relevant TOON table status after addressing each item
