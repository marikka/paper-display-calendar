use calendar::future_events;
fn main() {
    let ical_url = std::env::var("ICAL_URL").unwrap();
    for event in future_events(&ical_url).unwrap() {
        println!("{:#?} {:#?}", event.start, event.summary);
    }
}
