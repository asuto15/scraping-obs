use anyhow::Result;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, env, fs};

#[derive(Serialize, Deserialize)]
struct State {
    reservations: Vec<Reservation>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
struct Reservation {
    id: String,
    summary: String,
    created: DateTime<FixedOffset>,
    updated: DateTime<FixedOffset>,
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
}

#[derive(Deserialize)]
struct CalendarResponse {
    items: Vec<CalendarEvent>,
}

#[derive(Deserialize)]
struct CalendarEvent {
    id: String,
    summary: String,
    created: DateTime<FixedOffset>,
    updated: DateTime<FixedOffset>,
    #[serde(rename = "start")]
    start: TimeField,
    #[serde(rename = "end")]
    end: TimeField,
}

#[derive(Deserialize)]
struct TimeField {
    #[serde(rename = "dateTime")]
    date_time: DateTime<FixedOffset>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let jst: FixedOffset =
        FixedOffset::east_opt(9 * 3600).expect("Failed to create JST timezone offset");

    let calendar_url = env::var("CALENDAR_URL")?;
    let api_key = env::var("GOOGLE_API_KEY")?;
    let webhook_url = env::var("WEBHOOK_URL")?;
    let now_utc = Utc::now();
    let now = now_utc.with_timezone(&jst);
    let two_weeks = now + Duration::days(14);

    let client = Client::builder().build()?;
    let response: CalendarResponse = client
        .get(&calendar_url)
        .query(&[
            ("key", api_key.clone()),
            ("timeMin", now.to_rfc3339()),
            ("timeMax", two_weeks.to_rfc3339()),
            ("singleEvents", "true".to_string()),
            ("maxResults", "9999".to_string()),
            ("timeZone", "Asia/Tokyo".to_string()),
        ])
        .send()
        .await?
        .json()
        .await?;

    let new_reservations: Vec<Reservation> = response
        .items
        .into_iter()
        .map(|e| Reservation {
            id: e.id,
            summary: e.summary,
            created: e.created.with_timezone(&jst),
            updated: e.updated.with_timezone(&jst),
            start: e.start.date_time.with_timezone(&jst),
            end: e.end.date_time.with_timezone(&jst),
        })
        .collect();

    let state_path = "state.toml";
    if fs::metadata(state_path).is_err() {
        let state = State {
            reservations: new_reservations.clone(),
        };
        fs::write(state_path, toml::to_string_pretty(&state)?)?;
        return Ok(());
    }

    let old_state: State = toml::from_str(&fs::read_to_string(state_path)?)?;
    let old_reservations: Vec<Reservation> = old_state.reservations;

    let old_set: HashSet<_> = old_reservations.iter().cloned().collect();
    let new_set: HashSet<_> = new_reservations.iter().cloned().collect();
    let added: Vec<_> = new_set.difference(&old_set).cloned().collect();
    let removed: Vec<_> = old_set.difference(&new_set).cloned().collect();

    if !added.is_empty() || !removed.is_empty() {
        let mut content = String::new();
        if !added.is_empty() {
            content.push_str("**✨ 追加された予約**\n");
            for r in &added {
                content.push_str(&format!(
                    "- {} → {} ({})\n",
                    r.start.format("%Y-%m-%d %H:%M"),
                    r.end.format("%Y-%m-%d %H:%M"),
                    r.summary
                ));
            }
        }
        if !removed.is_empty() {
            content.push_str("**🗑️ 削除された予約**\n");
            for r in &removed {
                content.push_str(&format!(
                    "- {} → {} ({})\n",
                    r.start.format("%Y-%m-%d %H:%M"),
                    r.end.format("%Y-%m-%d %H:%M"),
                    r.summary
                ));
            }
        }
        client
            .post(&webhook_url)
            .json(&serde_json::json!({ "content": content }))
            .send()
            .await?;
    }

    let mut sorted_new_reservations = new_reservations;
    sorted_new_reservations.sort_by(|a, b| a.start.cmp(&b.start));
    let state = State {
        reservations: sorted_new_reservations,
    };
    fs::write(state_path, toml::to_string_pretty(&state)?)?;
    Ok(())
}
