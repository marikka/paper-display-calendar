use chrono::prelude::*;
use std::fs::File;
use std::io::BufReader;

struct Event {
    datetime: DateTime<Utc>,
    summary: String,
}

fn main() {
    let ical_url = std::env::var("ICAL_URL").unwrap();
    let response = reqwest::get(&ical_url).unwrap();
    let bf = BufReader::new(response);
    //let buf = BufReader::new(File::open("./basic.ics").unwrap());

    let reader = ical::IcalParser::new(bf);

    for line in reader.take(1) {
        let cal = line.unwrap();

        let mut events: Vec<Event> = cal
            .events
            .iter()
            .filter_map(|event| {
                let start: Option<&ical::property::Property> =
                    event.properties.iter().find(|p| p.name == "DTSTART");
                let summary: Option<&ical::property::Property> =
                    event.properties.iter().find(|p| p.name == "SUMMARY");

                if let (Some(start), Some(summary)) = (start, summary) {
                    if let (Some(start), Some(summary)) =
                        (start.value.as_ref(), summary.value.as_ref())
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

        let events_today = events
            .iter()
            .filter(|e| e.datetime.date() == Utc::now().date());

        for event in events_today {
            println!("{:#?} {:#?}", event.datetime, event.summary);
            println!("\n");
        }
    }
}
