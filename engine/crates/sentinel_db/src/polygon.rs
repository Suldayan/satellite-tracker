use postgis::ewkb::{Point, Polygon, LineString};

fn bbox_to_polygon(min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64) -> Polygon {
    let ring = LineString {
        points: vec![
            Point::new(min_lon, min_lat, None),
            Point::new(max_lon, min_lat, None),
            Point::new(max_lon, max_lat, None),
            Point::new(min_lon, max_lat, None),
            Point::new(min_lon, min_lat, None), 
        ],
    };

    Polygon {
        rings: vec![ring],
        srid: Some(4326),
    }
}