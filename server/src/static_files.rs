use once_cell::sync::Lazy;
use regex::Regex;

pub static CHAPTER_URL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"/manga/(?P<slug>[^/]+)/chapter-(?P<chapter>[\d.]+)/?").unwrap());

pub fn slug_to_title(slug: &str) -> String {
    slug.trim_end_matches("-all-chapters") // the quirk you flagged
        .replace('-', " ")
}
