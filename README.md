# Parley

A terminal-native tool for writing, publishing, and discussing RFDs (Requests for Discussion).

RFDs are authored in [Typst](https://typst.app/), pushed to a GitHub repository, and automatically
rendered by the Parley server. Teams read and discuss them entirely in the terminal — no browser
required.

See [Plan.md](Plan.md) for the full design document.

---

## Usage

```
# First run: start server and create bootstrap token
parley server start &
parley server bootstrap

# Launch the TUI
parley

# Token management
parley token create --user alice --role member
parley token list
parley token disable <id>
```

## Configuration

Parley looks for a config file at (in order):

- `/etc/parley/parley.toml`
- `./parley.toml`

Environment variables override config file values. Prefix: `PARLEY_`, double underscore for nesting.

```
PARLEY_SERVER__BIND_ADDRESS=0.0.0.0:7000
PARLEY_GIT__REMOTE_URL=https://github.com/org/rfds
PARLEY_WEBHOOK__SECRET=...
PARLEY_DEVELOPMENT__BYPASS_AUTH=true
```

## RFD Repository Layout

```
rfds/
└── rfd/
    └── 0001/
        ├── metadata.toml
        └── rfd.typ
```

`metadata.toml`:
```toml
title   = "My Proposal"
status  = "discussion"   # draft | discussion | accepted | rejected | abandoned
authors = ["alice"]
```

## Requirements

- `typst` binary on `$PATH` (or configured via `typst.binary_path`)
- GitHub repo with a webhook configured to `POST /webhook/github`
