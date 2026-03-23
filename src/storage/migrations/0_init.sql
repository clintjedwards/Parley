PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;

-- Tokens: primary identity primitive. Plaintext never stored — only SHA256 hash.
CREATE TABLE IF NOT EXISTS tokens (
    id       TEXT    NOT NULL,
    hash     TEXT    NOT NULL UNIQUE,
    created  TEXT    NOT NULL,
    expires  TEXT    NOT NULL,          -- epoch ms; "0" = never expires
    disabled INTEGER NOT NULL DEFAULT 0 CHECK (disabled IN (0,1)),
    user     TEXT    NOT NULL,
    roles    TEXT    NOT NULL,          -- JSON array of role IDs
    metadata TEXT    NOT NULL DEFAULT '{}',
    PRIMARY KEY (id)
) STRICT;
CREATE INDEX IF NOT EXISTS idx_tokens_hash ON tokens(hash);

-- Roles: named sets of permissions. system_role=1 rows are immutable.
CREATE TABLE IF NOT EXISTS roles (
    id          TEXT    NOT NULL,
    description TEXT    NOT NULL,
    permissions TEXT    NOT NULL,       -- JSON: [{resources:[...], actions:[...]}]
    system_role INTEGER NOT NULL DEFAULT 0 CHECK (system_role IN (0,1)),
    PRIMARY KEY (id)
) STRICT;

-- RFDs: one row per RFD. Content comes from git — this is the metadata index.
CREATE TABLE IF NOT EXISTS rfds (
    id      TEXT    NOT NULL,           -- zero-padded e.g. "0001"
    number  INTEGER NOT NULL UNIQUE CHECK (number > 0),
    title   TEXT    NOT NULL,
    status  TEXT    NOT NULL CHECK (status IN ('draft','discussion','accepted','rejected','abandoned')),
    authors TEXT    NOT NULL,           -- JSON array of strings
    created TEXT    NOT NULL,
    updated TEXT    NOT NULL,
    PRIMARY KEY (id)
) STRICT;

-- One row per git push that touched an RFD.
CREATE TABLE IF NOT EXISTS rfd_revisions (
    id             TEXT NOT NULL,
    rfd_id         TEXT NOT NULL REFERENCES rfds(id) ON DELETE CASCADE,
    commit_sha     TEXT NOT NULL,
    commit_message TEXT NOT NULL DEFAULT '',
    rendered_html  TEXT NOT NULL,       -- post-processed: data-pindex injected, styles scoped, sanitized
    title          TEXT NOT NULL,       -- metadata snapshot at this commit
    status         TEXT NOT NULL,
    authors        TEXT NOT NULL,       -- JSON array
    created        TEXT NOT NULL,
    PRIMARY KEY (id)
) STRICT;
CREATE INDEX IF NOT EXISTS idx_rfd_revisions_rfd_id ON rfd_revisions(rfd_id);

-- Discussion threads. Belong to an RFD; no sub-document anchoring.
CREATE TABLE IF NOT EXISTS threads (
    id          TEXT    NOT NULL,
    rfd_id      TEXT    NOT NULL REFERENCES rfds(id) ON DELETE CASCADE,
    resolved    INTEGER NOT NULL DEFAULT 0 CHECK (resolved IN (0,1)),
    resolved_by TEXT,                   -- token.user of resolver
    resolved_at TEXT,
    created_by  TEXT    NOT NULL,       -- token.user at creation time
    created     TEXT    NOT NULL,
    updated     TEXT    NOT NULL,
    PRIMARY KEY (id)
) STRICT;
CREATE INDEX IF NOT EXISTS idx_threads_rfd_id ON threads(rfd_id);

-- Flat messages within a thread. No nesting; use markdown >quote for attribution.
CREATE TABLE IF NOT EXISTS messages (
    id        TEXT NOT NULL,
    thread_id TEXT NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
    author    TEXT NOT NULL,            -- token.user (denormalized; survives token deletion)
    body      TEXT NOT NULL,            -- raw markdown
    body_html TEXT NOT NULL,            -- rendered + ammonia-sanitized HTML
    created   TEXT NOT NULL,
    updated   TEXT,
    PRIMARY KEY (id)
) STRICT;
CREATE INDEX IF NOT EXISTS idx_messages_thread_id ON messages(thread_id);

-- Append-only audit log. Never update or delete rows.
CREATE TABLE IF NOT EXISTS events (
    id        TEXT NOT NULL,
    kind      TEXT NOT NULL,
    actor     TEXT,                     -- token.user; null for system/webhook events
    rfd_id    TEXT REFERENCES rfds(id),
    thread_id TEXT REFERENCES threads(id),
    payload   TEXT NOT NULL DEFAULT '{}',
    created   TEXT NOT NULL,
    PRIMARY KEY (id)
) STRICT;
CREATE INDEX IF NOT EXISTS idx_events_rfd_id ON events(rfd_id);
CREATE INDEX IF NOT EXISTS idx_events_kind   ON events(kind);
