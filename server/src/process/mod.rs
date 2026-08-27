use strsim::jaro_winkler;

use crate::{
    db::{DB, types},
    parsing::{CHAPTER_URL_RE, slug_to_title},
};

const MATCH_THRESHOLD: f64 = 0.85;

pub fn resolve_event(db: &DB, event_id: i64) -> Result<(), rusqlite::Error> {
    let event = db.get_reading_event(event_id)?;

    let Some(caps) = CHAPTER_URL_RE.captures(&event.raw_url) else {
        tracing::warn!(
            "event {event_id}: url didn't match known pattern: {}",
            event.raw_url
        );
        return Ok(());
    };

    let title = slug_to_title(&caps["slug"]);
    let chapter: f64 = caps["chapter"].parse().unwrap_or(0.0);
    let now = chrono::Utc::now().timestamp();

    // 1. seen this exact domain+title combo before?
    let known_sources = db.get_series_sources_by_domain(&event.domain)?;
    let existing_match = known_sources
        .iter()
        .filter_map(|s| {
            let score = jaro_winkler(
                &s.site_title.as_deref().unwrap_or_default().to_lowercase(),
                &title.to_lowercase(),
            );
            (score >= MATCH_THRESHOLD).then_some((s, score))
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let series_id = if let Some((source, _)) = existing_match {
        source.series_id
    } else {
        // 2. known series, just new on this domain?
        let all_series = db.list_all_series()?;
        let title_match = all_series
            .iter()
            .map(|s| {
                (
                    s,
                    jaro_winkler(&s.canonical_title.to_lowercase(), &title.to_lowercase()),
                )
            })
            .filter(|(_, score)| *score >= MATCH_THRESHOLD)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let sid = match title_match {
            Some((series, _)) => series.id,
            // 3. genuinely new series
            None => db.create_series(&title)?,
        };

        // either way, this domain hasn't been linked to this series before — record it
        db.create_series_source(&types::NewSeriesSource {
            series_id: sid,
            domain: event.domain.clone(),
            site_title: Some(title.clone()),
            url_pattern: None, // global regex covers it; per-row pattern only needed for oddball sites later
        })?;

        sid
    };

    db.resolve_event(event_id, series_id)?;
    db.update_series_chapter(series_id, chapter, now)?;

    tracing::info!("event {event_id} resolved: series {series_id}, chapter {chapter}");
    Ok(())
}
