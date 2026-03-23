# Parley

A terminal-native tool for writing, publishing, and discussing RFDs (Requests for Discussion).
Designed for small engineering teams who live in the terminal.

---

## The Problem

RFD workflows typically require jumping between a git repo, a web UI for reading, and another
tool for discussion. For developers, this context switching is friction. The ideal tool would
let you write, read, and discuss RFDs without leaving the terminal.

Existing options fall short:
- **GitHub PRs**: discussions happen on raw Typst source, not the rendered document; UI is noisy
- **Web-based tools** (Notion, Confluence): not developer-centric, require browser
- **Plain git**: no structured discussion layer

---

## What Parley Is

A single Rust binary that runs two things:

1. **A server** — syncs RFDs from a GitHub repository, stores them rendered, and hosts a JSON
   API for discussions

2. **A TUI client** — a terminal UI (think lazygit) for reading RFDs and participating in
   discussions, with real-time updates when teammates comment

RFDs are written in Typst locally, pushed to GitHub, and the server picks them up automatically
via webhook. All reading and discussing happens in the TUI.

---

## RFD Lifecycle

```
Author writes RFD in Typst locally
        ↓
git push → GitHub
        ↓
GitHub webhook → Parley server
        ↓
Server pulls repo, compiles Typst → stores rendered content
        ↓
Team reads + discusses in TUI
        ↓
Status updated in metadata.toml (draft → discussion → accepted/rejected)
        ↓
git push → server picks up status change
```

---

## RFD Source Repository Layout

RFDs live in a separate GitHub repository:

```
rfds/
└── rfd/
    ├── 0001/
    │   ├── metadata.toml
    │   └── rfd.typ
    ├── 0002/
    │   ├── metadata.toml
    │   ├── rfd.typ
    │   └── assets/
    │       └── diagram.png
    ...
```

`metadata.toml`:
```toml
title   = "Distributed Tracing Strategy"
status  = "discussion"   # draft | discussion | accepted | rejected | abandoned
authors = ["clint"]
```

Status and authorship are managed by editing `metadata.toml` and pushing. The server tracks
all changes as immutable revisions.

---

## TUI Design

Vim-inspired. Keyboard driven. Three main views:

```
┌─────────────────────────────────────────────────┐
│  Parley  [RFDs]  [Discussions]          q:quit  │
├─────────────────────────────────────────────────┤
│                                                 │
│  RFD list / RFD content / Discussion threads    │
│                                                 │
└─────────────────────────────────────────────────┘
```

### RFD List view
- `j/k` — navigate
- `/` — search/filter
- `s` — filter by status
- `Enter` — open RFD
- `Tab` — switch to Discussions view

### RFD Content view
- `j/k` / arrow keys — scroll
- `c` — open a new discussion thread on this RFD
- `Tab` — switch to Discussions view for this RFD
- `q` — back to list

### Discussions view
- `j/k` — navigate threads
- `Enter` — expand thread to read messages
- `r` — reply (opens `$EDITOR` or inline compose)
- `R` — resolve/unresolve thread
- `n` — new thread
- `q` — back

### Discussion model
- Threads are flat — no nesting
- Quote with `>` in markdown for attribution (email-style)
- Each thread has a status: `open` | `resolved`
- Threads belong to an RFD; no sub-document anchoring
- Messages support markdown

### Real-time updates
A background task in the TUI maintains a WebSocket connection to the server. When a teammate
posts a message to a thread you have open, a notification appears in the status bar and the
thread updates without you doing anything.

---

## Server

The server is a background process (systemd service in production, `parley server` locally).
It exposes a JSON API consumed by the TUI client.

### Responsibilities
- Receive GitHub webhook on push
- `git pull` the RFD repository
- Compile changed `.typ` files via `typst compile --format html`
- Convert HTML to terminal-renderable text for TUI display
- Store RFDs, revisions, threads, and messages in SQLite
- Serve JSON API
- Push real-time updates to connected TUI clients via WebSocket

### Sync flow on push
1. Verify GitHub HMAC signature
2. Parse affected RFD paths from push payload
3. `git pull --ff-only` (fallback: `git fetch && git reset --hard origin/<branch>`)
4. For each changed RFD: read `metadata.toml`, compile Typst, store revision
5. Return 200

---

## Auth

Mirrors the auth model from Gofer. Token-based, no sessions.

- Tokens are random strings; only the SHA256 hash is stored in the DB
- Plaintext is shown exactly once on creation and never again
- Every API request sends `Authorization: Bearer <token>`
- Tokens have roles; roles have permissions over resources + actions

**System roles** (immutable):
- `bootstrap` — full access; created once via `parley server bootstrap`
- `admin` — manage tokens and all content
- `member` — read RFDs, create/edit own threads and messages
- `reader` — read-only

**Bootstrap**: on first run, `parley server bootstrap` prints a bootstrap token. All other tokens
are created through the API or TUI admin panel using that token.

**Development**: `bypass_auth = true` in config skips all auth checks.

---

## Stack

| Component | Choice |
|---|---|
| Language | Rust |
| HTTP framework | Dropshot (custom fork) |
| Database | SQLite via sqlx (split read/write pools) |
| TUI | ratatui |
| Real-time | WebSocket (server → TUI push) |
| Config | figment (TOML file + env vars) |
| RFD format | Typst (compiled via CLI) |
| Deployment | Single binary + systemd |

### Why Typst
- Expressive document format designed for technical writing
- Experimental HTML export (`typst compile --format html`) produces clean output
- The HTML is converted to terminal-renderable text for the TUI
- A `typst` library crate exists but requires implementing a `World` trait for font/file
  handling — not worth it; shelling out to the binary is simpler and more robust

### Why SQLite
- Zero ops: single file, no separate service
- Appropriate for a small team (2–10 people)
- Split connection pools: read pool (max 10 connections) + write pool (max 1) for safe concurrency

---

## Database Schema (summary)

| Table | Purpose |
|---|---|
| `tokens` | Auth tokens (hash only, never plaintext) |
| `roles` | Named permission sets; system roles are immutable |
| `rfds` | One row per RFD; metadata index |
| `rfd_revisions` | One row per git push; stores rendered content snapshot |
| `threads` | Discussion threads on an RFD |
| `messages` | Flat messages within a thread; markdown body |
| `events` | Append-only audit log of all actions |

All primary keys are UUIDv7. All timestamps are epoch milliseconds stored as TEXT.

---

## Project Layout

```
parley/
├── Cargo.toml
├── build.rs              -- BUILD_SEMVER + BUILD_COMMIT
├── config.toml           -- default config (embedded)
├── Plan.md               -- this document
├── systemd/
│   └── parley.service
└── src/
    ├── main.rs           -- tokio::main → cli dispatch
    ├── cli.rs            -- clap: server start/bootstrap, token management
    ├── conf.rs           -- figment config loading
    ├── errors.rs         -- StorageError + http_error! macro
    ├── models.rs         -- shared types
    ├── server/
    │   ├── mod.rs        -- AppState, route registration
    │   ├── api.rs        -- JSON API handlers
    │   ├── webhook.rs    -- GitHub webhook handler
    │   ├── ws.rs         -- WebSocket hub + per-client tasks
    │   └── permissioning.rs  -- roles, resources, actions
    ├── tui/
    │   ├── mod.rs        -- ratatui app loop, event handling
    │   ├── views/
    │   │   ├── rfd_list.rs
    │   │   ├── rfd_detail.rs
    │   │   └── discussions.rs
    │   └── ws_client.rs  -- background WebSocket listener → TUI event channel
    ├── git.rs            -- git pull, parse webhook payload paths
    ├── typst.rs          -- compile .typ → HTML, convert HTML → terminal text
    ├── markdown.rs       -- render + sanitize message bodies
    └── storage/
        ├── mod.rs        -- Db, connection pools, open_tx()
        ├── migrations/
        │   └── 0_init.sql
        ├── rfds.rs
        ├── revisions.rs
        ├── threads.rs
        ├── messages.rs
        ├── tokens.rs
        ├── roles.rs
        └── events.rs
```

---

## Configuration

```toml
[server]
bind_address = "0.0.0.0:7000"
storage_path = "/var/lib/parley/parley.db"

[git]
repo_path  = "/var/lib/parley/repo"
remote_url = ""
branch     = "main"

[webhook]
secret = ""   # must match GitHub repo webhook settings

[typst]
binary_path = "typst"

[log]
level  = "info"
pretty = false

[development]
bypass_auth = false
```

Config search order: embedded defaults → `/etc/parley/parley.toml` → `~/.config/parley/parley.toml`
→ `./parley.toml` → env vars (`PARLEY_` prefix, `__` for nesting).

---

## CLI

```
parley server start                       # start the API server
parley server bootstrap                   # print bootstrap token (first run only)
parley server sync                        # manually trigger full repo re-sync

parley token create --user alice --role member
parley token list
parley token disable <id>

parley                                    # launch the TUI (connects to configured server)
```

---

## What This Is Not

- Not a web application — no browser required
- Not a code review tool — RFDs are design documents, not diffs
- Not a replacement for git — git is still the source of truth for RFD content
- Not a general wiki — structure (numbered RFDs, explicit lifecycle) is intentional
