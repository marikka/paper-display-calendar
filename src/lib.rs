use chrono::prelude::*;
use std::io::BufReader;

pub struct Event {
    pub datetime: DateTime<Utc>,
    pub summary: String,
}

pub fn events(ical_url: &str) -> Result<Vec<Event>, reqwest::Error> {
    let response = reqwest::get(ical_url)?;
    let bf = BufReader::new(response);
    let mut reader = ical::IcalParser::new(bf);
    let cal = reader.next().unwrap().unwrap();
    let mut events: Vec<Event> = cal
        .events
        .iter()
        .filter_map(|event| {
            let start: Option<&ical::property::Property> =
                event.properties.iter().find(|p| p.name == "DTSTART");
            let summary: Option<&ical::property::Property> =
                event.properties.iter().find(|p| p.name == "SUMMARY");

            if let (Some(start), Some(summary)) = (start, summary) {
                if let (Some(start), Some(summary)) = (start.value.as_ref(), summary.value.as_ref())
                {
                    if let Ok(dt) = Utc.datetime_from_str(&start, "%Y%m%dT%H%M%SZ") {
                        return Some(Event {
                            datetime: dt,
                            summary: summary.to_string(),
                        });
                    }
                }
            }
            None
        })
        .collect();

    events.sort_by(|a, b| a.datetime.cmp(&b.datetime));
    Ok(events)
}

pub fn events_today(ical_url: &str) -> Result<Vec<Event>, reqwest::Error> {
    Ok(events(ical_url)?
        .into_iter()
        .filter(|e| e.datetime.date() == Utc::now().date())
        .collect())
}

pub fn future_events(ical_url: &str) -> Result<Vec<Event>, reqwest::Error> {
    Ok(events(ical_url)?.into_iter().collect())
}
