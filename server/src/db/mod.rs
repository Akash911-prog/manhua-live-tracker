pub mod types;

use std::sync::Mutex;

use rusqlite::Connection;

pub struct DB {
    conn: Mutex<Connection>,
}

impl DB {
    // generic: run a query expecting exactly one row
    fn query_one<T>(
        &self,
        sql: &str,
        params: impl rusqlite::Params,
        map: impl FnOnce(&rusqlite::Row) -> rusqlite::Result<T>,
    ) -> Result<T, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(sql, params, map)
    }

    // generic: run a query expecting zero or more rows
    fn query_many<T>(
        &self,
        sql: &str,
        params: impl rusqlite::Params,
        map: impl Fn(&rusqlite::Row) -> rusqlite::Result<T>,
    ) -> Result<Vec<T>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        stmt.query_map(params, map)?.collect()
    }

    // generic: insert/update/delete, returns rows affected
    fn exec(&self, sql: &str, params: impl rusqlite::Params) -> Result<usize, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(sql, params)
    }
}

impl DB {
    pub fn init(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch(include_str!("../../db/migrations/001_init.sql"))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn get_series(&self, id: i64) -> Result<types::Series, rusqlite::Error> {
        self.query_one(
            "SELECT * FROM series WHERE id = ?",
            [id],
            types::Series::from_row,
        )
    }

    pub fn get_series_sources(
        &self,
        series_id: i64,
    ) -> Result<Vec<types::SeriesSource>, rusqlite::Error> {
        self.query_many(
            "SELECT * FROM series_sources WHERE series_id = ?",
            [series_id],
            types::SeriesSource::from_row,
        )
    }

    pub fn get_reading_event(&self, id: i64) -> Result<types::ReadingEvent, rusqlite::Error> {
        self.query_one(
            "SELECT * FROM reading_events WHERE id = ?",
            [id],
            types::ReadingEvent::from_row,
        )
    }

    pub fn get_series_sources_by_domain(
        &self,
        domain: &str,
    ) -> Result<Vec<types::SeriesSource>, rusqlite::Error> {
        self.query_many(
            "SELECT * FROM series_sources WHERE domain = ?",
            [domain],
            types::SeriesSource::from_row,
        )
    }

    pub fn list_all_series(&self) -> Result<Vec<types::Series>, rusqlite::Error> {
        self.query_many("SELECT * FROM series", [], types::Series::from_row)
    }

    pub fn list_series(&self) -> Result<Vec<types::Series>, rusqlite::Error> {
        self.query_many(
            "SELECT * FROM series ORDER BY last_updated_at DESC",
            [],
            types::Series::from_row,
        )
    }

    pub fn insert_event(&self, e: &types::NewEvent) -> Result<i64, rusqlite::Error> {
        self.exec(
            "INSERT INTO reading_events (source, domain, raw_url, detected_at, synced) VALUES (?1, ?2, ?3, ?4, 0)",
            rusqlite::params![e.source, e.domain, e.raw_url, e.detected_at],
        )?;
        Ok(self.last_insert_id())
    }

    pub fn create_series(&self, title: &str) -> Result<i64, rusqlite::Error> {
        self.exec(
            "INSERT INTO series (canonical_title, current_chapter, last_updated_at, status) VALUES (?1, NULL, NULL, 'reading')",
            [title],
        )?;
        Ok(self.last_insert_id())
    }

    fn last_insert_id(&self) -> i64 {
        self.conn.lock().unwrap().last_insert_rowid()
    }
}

impl DB {
    // --- creates ---

    pub fn create_series_source(&self, s: &types::NewSeriesSource) -> Result<i64, rusqlite::Error> {
        self.exec(
            "INSERT INTO series_sources (series_id, domain, site_title, url_pattern) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![s.series_id, s.domain, s.site_title, s.url_pattern],
        )?;
        Ok(self.last_insert_id())
    }

    // --- edits ---

    pub fn update_series_chapter(
        &self,
        series_id: i64,
        chapter: f64,
        now: i64,
    ) -> Result<bool, rusqlite::Error> {
        let rows = self.exec(
            "UPDATE series SET current_chapter = ?1, last_updated_at = ?2
             WHERE id = ?3 AND (current_chapter IS NULL OR current_chapter < ?1)",
            rusqlite::params![chapter, now, series_id],
        )?;
        Ok(rows > 0)
    }

    pub fn update_series_status(
        &self,
        series_id: i64,
        status: &str,
    ) -> Result<(), rusqlite::Error> {
        self.exec(
            "UPDATE series SET status = ?1 WHERE id = ?2",
            rusqlite::params![status, series_id],
        )?;
        Ok(())
    }

    pub fn update_series_source(
        &self,
        source_id: i64,
        s: &types::NewSeriesSource,
    ) -> Result<(), rusqlite::Error> {
        self.exec(
            "UPDATE series_sources SET domain = ?1, site_title = ?2, url_pattern = ?3 WHERE id = ?4",
            rusqlite::params![s.domain, s.site_title, s.url_pattern, source_id],
        )?;
        Ok(())
    }

    pub fn resolve_event(&self, event_id: i64, series_id: i64) -> Result<(), rusqlite::Error> {
        self.exec(
            "UPDATE reading_events SET series_id = ?1 WHERE id = ?2",
            rusqlite::params![series_id, event_id],
        )?;
        Ok(())
    }

    pub fn mark_events_synced(&self, event_ids: &[i64]) -> Result<(), rusqlite::Error> {
        let placeholders = event_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "UPDATE reading_events SET synced = 1 WHERE id IN ({})",
            placeholders
        );
        self.exec(&sql, rusqlite::params_from_iter(event_ids.iter()))?;
        Ok(())
    }

    // called by the TUI to queue a URL for a specific phone
    pub fn create_pending_open(
        &self,
        p: &types::NewPendingOpen,
        now: i64,
    ) -> Result<i64, rusqlite::Error> {
        self.exec(
            "INSERT INTO pending_opens (target_device, url, created_at, delivered) VALUES (?1, ?2, ?3, 0)",
            rusqlite::params![p.target_device, p.url, now],
        )?;
        Ok(self.last_insert_id())
    }

    // called by a phone's sync job, filtered to just its own device id
    pub fn get_pending_opens(
        &self,
        target_device: &str,
    ) -> Result<Vec<types::PendingOpen>, rusqlite::Error> {
        self.query_many(
            "SELECT * FROM pending_opens WHERE target_device = ?1 AND delivered = 0 ORDER BY created_at ASC",
            [target_device],
            types::PendingOpen::from_row,
        )
    }

    // called by the phone once it's actually opened the URL
    pub fn mark_pending_open_delivered(&self, id: i64) -> Result<(), rusqlite::Error> {
        self.exec(
            "UPDATE pending_opens SET delivered = 1 WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }
}
