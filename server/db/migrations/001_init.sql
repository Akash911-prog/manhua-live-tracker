CREATE TABLE IF NOT EXISTS series (
    id INTEGER PRIMARY KEY,
    canonical_title TEXT NOT NULL,
    current_chapter REAL,
    last_updated_at INTEGER,
    status TEXT DEFAULT 'reading'
);

CREATE TABLE IF NOT EXISTS series_sources (
    id INTEGER PRIMARY KEY,
    series_id INTEGER REFERENCES series(id),
    domain TEXT NOT NULL,
    site_title TEXT,
    url_pattern TEXT
);

CREATE TABLE IF NOT EXISTS reading_events (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL,
    domain TEXT NOT NULL,
    raw_url TEXT NOT NULL,
    detected_at INTEGER,
    synced INTEGER DEFAULT 0,
    series_id INTEGER REFERENCES series(id)
);

-- URLs queued by the TUI to be opened on a specific phone (send-to-phone feature).
-- target_device matches the same identifier reading_events.source uses ("phone-1", "phone-2").
CREATE TABLE IF NOT EXISTS pending_opens (
    id INTEGER PRIMARY KEY,
    target_device TEXT NOT NULL,
    url TEXT NOT NULL,
    created_at INTEGER,
    delivered INTEGER DEFAULT 0
);