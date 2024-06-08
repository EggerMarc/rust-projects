
use serde::Deserialize;
use std::fs::File;


#[derive(Debug, Deserialize)]
struct Feature {
    location: Location,
    name: String
}

#[derive(Debug, Deserialize)]
struct Location {
    latitude: i32,
    longitude: i32 
}

#[allow(dead_code)]
pub fn load() -> Vec<crate::proto::Feature> {
    let dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR"), "data"]);
    let file = File::open(dir.join("route_guide_data.json")).expect("Something happened, can't find data");

    let decoded: Vec<Feature> = serde_json::from_reader(&file).expect("Serde json failed to serialize");

    decoded.into_iter().map(|entry| crate::proto::Feature {
        name: entry.name,
        location: Some(crate::proto::Point {
            latitude: entry.location.latitude,
            longitude: entry.location.longitude
        })
    }).collect()
}
