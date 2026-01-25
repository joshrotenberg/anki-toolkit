//! SQLite schema for Anki .apkg files.
//!
//! Uses schema version 11 for maximum compatibility.

/// SQL to create the database schema.
pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS col (
    id              INTEGER PRIMARY KEY,
    crt             INTEGER NOT NULL,
    mod             INTEGER NOT NULL,
    scm             INTEGER NOT NULL,
    ver             INTEGER NOT NULL,
    dty             INTEGER NOT NULL,
    usn             INTEGER NOT NULL,
    ls              INTEGER NOT NULL,
    conf            TEXT NOT NULL,
    models          TEXT NOT NULL,
    decks           TEXT NOT NULL,
    dconf           TEXT NOT NULL,
    tags            TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notes (
    id              INTEGER PRIMARY KEY,
    guid            TEXT NOT NULL,
    mid             INTEGER NOT NULL,
    mod             INTEGER NOT NULL,
    usn             INTEGER NOT NULL,
    tags            TEXT NOT NULL,
    flds            TEXT NOT NULL,
    sfld            INTEGER NOT NULL,
    csum            INTEGER NOT NULL,
    flags           INTEGER NOT NULL,
    data            TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cards (
    id              INTEGER PRIMARY KEY,
    nid             INTEGER NOT NULL,
    did             INTEGER NOT NULL,
    ord             INTEGER NOT NULL,
    mod             INTEGER NOT NULL,
    usn             INTEGER NOT NULL,
    type            INTEGER NOT NULL,
    queue           INTEGER NOT NULL,
    due             INTEGER NOT NULL,
    ivl             INTEGER NOT NULL,
    factor          INTEGER NOT NULL,
    reps            INTEGER NOT NULL,
    lapses          INTEGER NOT NULL,
    left            INTEGER NOT NULL,
    odue            INTEGER NOT NULL,
    odid            INTEGER NOT NULL,
    flags           INTEGER NOT NULL,
    data            TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS revlog (
    id              INTEGER PRIMARY KEY,
    cid             INTEGER NOT NULL,
    usn             INTEGER NOT NULL,
    ease            INTEGER NOT NULL,
    ivl             INTEGER NOT NULL,
    lastIvl         INTEGER NOT NULL,
    factor          INTEGER NOT NULL,
    time            INTEGER NOT NULL,
    type            INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS graves (
    usn             INTEGER NOT NULL,
    oid             INTEGER NOT NULL,
    type            INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS ix_notes_usn ON notes (usn);
CREATE INDEX IF NOT EXISTS ix_cards_usn ON cards (usn);
CREATE INDEX IF NOT EXISTS ix_revlog_usn ON revlog (usn);
CREATE INDEX IF NOT EXISTS ix_cards_nid ON cards (nid);
CREATE INDEX IF NOT EXISTS ix_cards_sched ON cards (did, queue, due);
CREATE INDEX IF NOT EXISTS ix_revlog_cid ON revlog (cid);
CREATE INDEX IF NOT EXISTS ix_notes_csum ON notes (csum);
"#;

/// Default collection configuration JSON.
pub const DEFAULT_CONF: &str = r#"{
    "activeDecks": [1],
    "curDeck": 1,
    "newSpread": 0,
    "collapseTime": 1200,
    "timeLim": 0,
    "estTimes": true,
    "dueCounts": true,
    "curModel": null,
    "nextPos": 1,
    "sortType": "noteFld",
    "sortBackwards": false,
    "addToCur": true
}"#;

/// Default deck configuration JSON.
pub const DEFAULT_DCONF: &str = r#"{
    "1": {
        "id": 1,
        "mod": 0,
        "name": "Default",
        "usn": 0,
        "maxTaken": 60,
        "autoplay": true,
        "timer": 0,
        "replayq": true,
        "new": {
            "bury": true,
            "delays": [1, 10],
            "initialFactor": 2500,
            "ints": [1, 4, 7],
            "order": 1,
            "perDay": 20,
            "separate": true
        },
        "rev": {
            "bury": true,
            "ease4": 1.3,
            "fuzz": 0.05,
            "ivlFct": 1,
            "maxIvl": 36500,
            "perDay": 100,
            "hardFactor": 1.2
        },
        "lapse": {
            "delays": [10],
            "leechAction": 0,
            "leechFails": 8,
            "minInt": 1,
            "mult": 0
        },
        "dyn": false
    }
}"#;

/// Field separator character (ASCII unit separator).
pub const FIELD_SEPARATOR: char = '\x1f';
