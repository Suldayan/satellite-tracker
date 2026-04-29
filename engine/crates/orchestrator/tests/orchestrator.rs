use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use predictor::SatellitePassEvent;
use chrono::Utc;

fn mock_event() -> SatellitePassEvent {
    SatellitePassEvent {
        satellite_id: "TEST-SAT".into(),
        pass_start: Utc::now(),
        pass_end: Utc::now() + chrono::Duration::minutes(5),
        max_elevation_deg: 45.0,
        min_lon: -122.95, max_lon: -122.65,
        min_lat: 49.05, max_lat: 49.35,
    }
}

#[test]
fn channel_delivers_event_to_receiver() {
    let (tx, rx) = mpsc::channel::<SatellitePassEvent>();

    thread::spawn(move || {
        tx.send(mock_event()).expect("send failed");
    });

    let event = rx.recv_timeout(Duration::from_secs(2))
        .expect("timed out waiting for event");

    assert_eq!(event.satellite_id, "TEST-SAT");
    assert!(event.max_elevation_deg > 10.0);
}

#[test]
fn channel_closes_cleanly_when_sender_drops() {
    let (tx, rx) = mpsc::channel::<SatellitePassEvent>();
    drop(tx);
    
    assert!(rx.recv().is_err());
}

#[test]
fn multiple_events_arrive_in_order() {
    let (tx, rx) = mpsc::channel::<SatellitePassEvent>();

    for i in 0..5 {
        let mut event = mock_event();
        event.satellite_id = format!("SAT-{i}");
        tx.send(event).unwrap();
    }
    drop(tx);

    let ids: Vec<String> = rx.iter()
        .map(|e| e.satellite_id.clone())
        .collect();

    assert_eq!(ids, vec!["SAT-0","SAT-1","SAT-2","SAT-3","SAT-4"]);
}