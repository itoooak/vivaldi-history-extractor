use chrono::{DateTime, Duration, TimeZone, Utc};

#[derive(Debug, serde::Serialize)]
pub struct SearchRecord {
    normalized_term: String,
    visit_count: u32,
    last_visit_time: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct VisitRecord {
    url: String,
    visit_time: DateTime<Utc>,
    visit_duration: u64,
}

fn convert_timestamp(ts: i64) -> Option<DateTime<Utc>> {
    // Chromiumのtimestampは1601-01-01から
    // cf. https://www.epochconverter.com/webkit
    let epoch_start = Utc.with_ymd_and_hms(1601, 1, 1, 0, 0, 0).unwrap();
    let delta = Duration::microseconds(ts);
    epoch_start.checked_add_signed(delta)
}

pub fn get_search_records(
    conn: rusqlite::Connection,
) -> Result<Vec<SearchRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "
            SELECT normalized_term, SUM(visit_count), MAX(last_visit_time)
            FROM keyword_search_terms
            JOIN urls
            ON keyword_search_terms.url_id = urls.id
            WHERE visit_count > 0
            GROUP BY normalized_term
            ORDER BY MAX(last_visit_time) DESC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(SearchRecord {
            normalized_term: row.get(0)?,
            visit_count: row.get(1)?,
            last_visit_time: convert_timestamp(row.get(2)?).unwrap(),
        })
    })?;

    Ok(rows.filter_map(|x| x.ok()).collect())
}

pub fn get_visit_records(conn: rusqlite::Connection) -> Result<Vec<VisitRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "
            SELECT urls.url, visit_time, visit_duration
            FROM visits
            JOIN urls
            ON visits.url = urls.id
            ORDER BY visit_duration DESC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(VisitRecord {
            url: row.get(0)?,
            visit_time: convert_timestamp(row.get(1)?).unwrap(),
            visit_duration: row.get(2)?,
        })
    })?;

    Ok(rows.filter_map(|x| x.ok()).collect())
}
