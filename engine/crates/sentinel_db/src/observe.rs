use std::sync::mpsc::Receiver;
use std::thread;
use sentinel_events::Event;
use connection::connect;
use polygon::bbox_to_polygon;

pub fn listen(rx: Receiver<Event>) {
    thread::spawn(move || {
        for event in rx {
            match event {
                Event::PipelineFinished(result) => {
                    match result {
                        Ok(Some(path)) => {
                            println!("NDVI saved at {path}");
                            // insert into DB here
                            let polygon = bbox_to_polygon(
                                result.min_lon, 
                                result.min_lat, 
                                result.max_lon,
                                result.max_lat);
                        }

                        // ndvi result stuff and postgis insert

                        Ok(None) => {
                            println!("No imagery available");
                        }
                        Err(err) => {
                            println!("Pipeline failed: {err}");
                        }
                    }
                }
                _ => {}
            }
        }
    });
}
