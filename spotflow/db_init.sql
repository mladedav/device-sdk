PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS Messages (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    site_id             TEXT,
    stream_group        TEXT,
    stream              TEXT,
    batch_id            TEXT,
    message_id          TEXT,
    content             BLOB NOT NULL,
    close_option        TEXT NOT NULL,
    compression         TEXT NOT NULL,
    batch_slice_id      TEXT,
    chunk_id            TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS CloudToDeviceMessages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content BLOB NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS CloudToDeviceProperties (
    message_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,

    UNIQUE(message_id, key),
    FOREIGN KEY(message_id) REFERENCES CloudToDeviceMessages(id)
) STRICT;

CREATE TABLE IF NOT EXISTS Twins (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    type                TEXT NOT NULL,
    properties          TEXT NOT NULL -- JSON
) STRICT;

CREATE TABLE IF NOT EXISTS ReportedPropertiesUpdates (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    update_type         TEXT NOT NULL, -- UpdateType enum
    patch               TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS _Channel (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    type                TEXT NOT NULL,
    value               TEXT NOT NULL -- JSON
) STRICT;

CREATE TABLE IF NOT EXISTS SdkConfiguration (
    id                  INTEGER PRIMARY KEY,
    db_version          TEXT NOT NULL,
    instance_url        TEXT NOT NULL,
    provisioning_token  TEXT NOT NULL,
    registration_token  TEXT NOT NULL,
    rt_expiration       TEXT, -- DATETIME
    requested_device_id TEXT,
    workspace_id        TEXT NOT NULL,
    device_id           TEXT NOT NULL
) STRICT;
