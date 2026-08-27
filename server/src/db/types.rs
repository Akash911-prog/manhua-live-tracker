#[derive(Debug, serde::Serialize)]
pub struct Series {
    pub id: i64,
    pub canonical_title: String,
    pub current_chapter: Option<f64>,
    pub last_updated_at: Option<i64>,
    pub status: String,
}

impl Series {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            canonical_title: row.get(1)?,
            current_chapter: row.get(2)?,
            last_updated_at: row.get(3)?,
            status: row.get(4)?,
        })
    }
}

pub struct SeriesSource {
    pub id: i64,
    pub series_id: i64,
    pub domain: String,
    pub site_title: Option<String>,
    pub url_pattern: Option<String>,
}

impl SeriesSource {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            series_id: row.get(1)?,
            domain: row.get(2)?,
            site_title: row.get(3)?,
            url_pattern: row.get(4)?,
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ReadingEvent {
    pub id: i64,
    pub source: String,
    pub domain: String,
    pub raw_url: String,
    pub detected_at: Option<i64>,
    pub synced: Option<i64>,
    pub series_id: Option<i64>,
}

impl ReadingEvent {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            source: row.get(1)?,
            domain: row.get(2)?,
            raw_url: row.get(3)?,
            detected_at: row.get(4)?,
            synced: row.get(5)?,
            series_id: row.get(6)?,
        })
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct NewEvent {
    pub source: String, // "phone-1", "phone-2", "pc-extension"
    pub domain: String,
    pub raw_url: String,
    pub detected_at: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct NewSeriesSource {
    pub series_id: i64,
    pub domain: String,
    pub site_title: Option<String>,
    pub url_pattern: Option<String>,
}
