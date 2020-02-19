use calendar::events_today;
fn main() {
    let ical_url = std::env::var("ICAL_URL").unwrap();
    for event in events_today(&ical_url).unwrap() {
        println!("{:#?} {:#?}", event.start, event.summary);
    }
}
