CREATE TABLE IF NOT EXISTS users (
    id                  TEXT    NOT NULL,
    name                TEXT    NOT NULL,
    created             TEXT    NOT NULL,
    modified            TEXT    NOT NULL,
    PRIMARY KEY (id)
) STRICT;
