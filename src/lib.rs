use chrono::prelude::*;
use std::io::BufReader;

#[derive(Debug, PartialEq)]
pub struct Event {
    pub start: DateTime<Utc>,
    pub summary: String,
}

pub fn events_from_ical_url(ical_url: &str) -> Result<Vec<Event>, reqwest::Error> {
    let response = reqwest::get(ical_url)?;
    let bf = BufReader::new(response);
    let mut reader = ical::IcalParser::new(bf);
    let cal = reader.next().unwrap().unwrap();
    let mut events: Vec<Event> = parse_events(cal.events);
    events.sort_by(|a, b| a.start.cmp(&b.start));
    Ok(events)
}

pub fn events_from_ical_urls(ical_urls: Vec<&str>) -> Result<Vec<Event>, reqwest::Error> {
    let mut events: Vec<Event> = ical_urls
        .iter()
        .flat_map(|url| events_from_ical_url(url).unwrap())
        .collect();
    events.sort_by(|a, b| a.start.cmp(&b.start));
    Ok(events)
}

fn parse_events(events: Vec<ical::parser::ical::component::IcalEvent>) -> Vec<Event> {
    return events
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
                            start: dt,
                            summary: summary.to_string(),
                        });
                    }
                }
            }
            None
        })
        .collect();
}

pub fn future_events(ical_url: &str) -> Result<Vec<Event>, reqwest::Error> {
    Ok(events_from_ical_url(ical_url)?
        .into_iter()
        .filter(|e| e.start.date() >= Utc::now().date())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use ical::parser::ical::component::IcalEvent;
    use ical::property::Property;

    #[test]
    fn parse_events_works() {
        let events = vec![IcalEvent {
            alarms: vec![],
            properties: vec![
                Property {
                    name: String::from("DTSTART"),
                    params: None,
                    value: Some(String::from("20200121T200000Z")),
                },
                Property {
                    name: String::from("SUMMARY"),
                    params: None,
                    value: Some(String::from("foo")),
                },
            ],
        }];

        assert_eq!(
            parse_events(events),
            vec![Event {
                start: Utc.ymd(2020, 01, 21).and_hms(20, 0, 0),
                summary: String::from("foo")
            }]
        );
    }
}
