use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Feature {
    location: Location,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Location {
    latitude: i32,
    longitude: i32,
}

#[allow(dead_code)]
pub fn load() -> Result<Vec<crate::proto::Feature>, Box<dyn Error>> {
    let data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/route_guide_db.json");
    let file = File::open(&data_path)?;

    let features: Vec<Feature> = serde_json::from_reader(file)?;

    Ok(features
        .into_iter()
        .map(|entry| crate::proto::Feature {
            name: entry.name.to_string(),
            location: Some(crate::proto::Point {
                latitude: entry.location.latitude,
                longitude: entry.location.longitude,
            }),
        })
        .collect())
}
