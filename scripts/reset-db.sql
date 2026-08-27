-- Manual dev-only reset. NOT a migration — do not include this via include_str!
-- or run it against a db you care about. Run by hand: sqlite3 manhua.db < reset-db.sql
DROP TABLE reading_events;
DROP TABLE series_sources;
DROP TABLE series;