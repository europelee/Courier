# Design: CLAUDE.md + Developer Rules Documentation

**Date:** 2026-04-18  
**Project:** Courier (内网穿透隧道管理系统)  
**Goal:** Add CLAUDE.md and structured rule files to guide Claude Code when working in this project

---

## Context

Courier is a Rust + Vue3 intranet tunnel management system (similar to ngrok). It uses:
- **Backend:** Rust workspace (shared/server/client crates), Axum, SQLite, WebSocket
- **Frontend:** Vue3 + TypeScript (minimal: App.vue, main.ts, src/api/)
- **No existing CLAUDE.md or .claude/ directory**

The audience for these documents is **Claude Code (AI assistant)**, not human developers. The goal is to reduce repeated context-setting and enforce consistent behavior across sessions.

---

## File Structure

```
CLAUDE.md                          ← Entry point (≤150 lines)
.claude/
  rules/
    01-architecture.md             ← Module responsibilities, data flow
    02-rust-conventions.md         ← Rust coding standards
    03-frontend-conventions.md     ← Vue3/TypeScript standards
    04-git-workflow.md             ← Branch, commit, PR rules
    05-testing.md                  ← TDD requirements, test strategy
    06-constraints.md              ← Hard prohibitions (must not do list)
```

---

## CLAUDE.md — Entry Point

Sections:
1. One-line project description
2. Tech stack overview (Rust 1.94+, Axum 0.7, SQLite, Vue3, TypeScript)
3. Common commands (build, test, run, lint)
4. Rules index (pointers to .claude/rules/ files with one-line summaries)

Kept under 150 lines — loaded every session, must stay lightweight.

---

## Rule Files

### 01-architecture.md
- Workspace layout: `shared/`, `server/`, `client/`, `web/`
- Server module responsibilities: auth.rs (JWT), db.rs (SQLite), handlers.rs (REST), websocket.rs (tunnel), validation.rs, errors.rs
- HTTP request flow: request → handlers.rs → auth.rs → db.rs → response
- WebSocket tunnel flow: client connects → websocket.rs → forward to local service
- Key ports: 8080 (HTTP), 8443 (HTTPS), 3000 (frontend dev)

### 02-rust-conventions.md
- Error handling: `thiserror` for library errors, `anyhow` for application layer
- **Forbidden:** `.unwrap()` in non-test code; use `?` or explicit handling
- Async: always `tokio`, never `std::thread::sleep`
- Naming: `snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants
- Logging: `tracing` macros (`info!`, `warn!`, `error!`), never `println!`
- Keep modules focused: one clear responsibility per file

### 03-frontend-conventions.md
- Vue3 Composition API only — no Options API
- TypeScript strict mode — no `any` type
- All API calls go through `src/api/` — no inline fetch calls in components
- Use `const` over `let` where possible

### 04-git-workflow.md
- Branch naming: `feat/<name>`, `fix/<name>`, `chore/<name>`
- Commit format: Conventional Commits (`feat: add X`, `fix: correct Y`, `chore: update Z`)
- **Forbidden:** `git push origin main` directly — all changes via PR
- PR must pass all tests before merge

### 05-testing.md
- TDD flow: write failing test → implement → pass → refactor
- Rust: `cargo test --workspace` for all tests; integration tests in `tests/`
- Frontend: Vue Test Utils for component tests
- Every new feature must have test coverage
- Every bugfix must have a regression test
- Never mark a task complete if tests are failing

### 06-constraints.md (Hard Prohibitions)
- **No unrequested refactoring** — only modify what was asked; do not "improve" surrounding code
- **No skipping tests** — implement nothing without tests first
- **No direct push to main** — always use PR workflow
- **No large feature without design doc** — features >50 lines of new code require a written design first
- **No `.unwrap()` in production code** — test files are the only exception
- **No `console.log` left in committed frontend code**
- **No adding dependencies without noting them** — always mention new Cargo.toml or package.json additions

---

## Decisions

| Decision | Choice | Reason |
|---|---|---|
| Audience | Claude Code (AI) | Human docs already exist (CONTRIBUTING.md, README.md) |
| Structure | CLAUDE.md + .claude/rules/ | CLAUDE.md stays lightweight; rules loaded on demand |
| Language | Chinese + English mixed | Project docs are in Chinese; rule files in English for precision |
| Rule file naming | Numbered prefix (01-, 02-…) | Ensures consistent ordering, easy to reference |

---

## Out of Scope

- Replacing or modifying existing CONTRIBUTING.md / README.md / SECURITY.md
- Adding CI/CD configuration
- Adding linting config files (clippy.toml, eslint config)
