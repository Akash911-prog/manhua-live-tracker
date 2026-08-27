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
    raw_title TEXT,
    chapter REAL,
    detected_at INTEGER,
    synced INTEGER DEFAULT 0
);