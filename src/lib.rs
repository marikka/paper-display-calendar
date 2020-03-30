use chrono::prelude::*;
use chrono::Utc;
use chrono_tz::Tz;
use ical::property::Property;
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

fn try_read_timezone(params: Option<&Vec<(String, Vec<String>)>>) -> Option<Tz> {
    let tzid_param = &params?.iter().find(|param| param.0 == "TZID")?.1;
    tzid_param.first()?.parse::<Tz>().ok()
}

fn try_read_utc_datetime(start_property: &Property) -> Option<DateTime<Utc>> {
    Utc.datetime_from_str(start_property.value.as_ref()?, "%Y%m%dT%H%M%SZ")
        .ok()
}

fn try_read_timezone_datetime(start_property: &Property) -> Option<DateTime<Utc>> {
    let timezone = try_read_timezone(start_property.params.as_ref())?;
    let datetime = timezone
        .datetime_from_str(start_property.value.as_ref()?, "%Y%m%dT%H%M%S")
        .ok()?;
    let utc = datetime.with_timezone(&Utc);
    Some(utc)
}

fn parse_events(events: Vec<ical::parser::ical::component::IcalEvent>) -> Vec<Event> {
    return events
        .iter()
        .filter_map(|event| {
            let start: &ical::property::Property =
                event.properties.iter().find(|p| p.name == "DTSTART")?;
            let summary: &ical::property::Property =
                event.properties.iter().find(|p| p.name == "SUMMARY")?;
            let summary_value = summary.value.as_ref()?;
            let datetime =
                try_read_utc_datetime(start).or_else(|| try_read_timezone_datetime(start))?;
            Some(Event {
                start: datetime,
                summary: summary_value.to_string(),
            })
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

    #[test]
    fn parse_events_with_timezone() {
        let events = vec![IcalEvent {
            alarms: vec![],
            properties: vec![
                Property {
                    name: String::from("DTSTART"),
                    params: Some(vec![(
                        String::from("TZID"),
                        vec![String::from("America/New_York")],
                    )]),
                    value: Some(String::from("20200110T150000")),
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
                start: Utc.ymd(2020, 01, 10).and_hms(20, 0, 0),
                summary: String::from("foo")
            }]
        );
    }
}
