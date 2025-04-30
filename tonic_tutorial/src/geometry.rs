//! Pure geometry helpers — no async, no gRPC.

use once_cell::sync::OnceCell;
use rstar::{RTreeObject, AABB};
use std::hash::{Hash, Hasher};

use crate::proto::Point;

/// Latitude/longitude in radians, plus the original protobuf `Point`.
#[derive(Debug, Copy, Clone)]
pub struct GeoPoint {
    pub pb: Point,
    pub lat_rad: f64,
    pub lon_rad: f64,
}

impl From<Point> for GeoPoint {
    fn from(pb: Point) -> Self {
        const SCALE: f64 = 1e7;
        Self {
            lat_rad: (pb.latitude as f64 / SCALE).to_radians(),
            lon_rad: (pb.longitude as f64 / SCALE).to_radians(),
            pb,
        }
    }
}

/// Global, immutable coordinate table every R-tree leaf indexes into.
pub static GEO_POINTS: OnceCell<Vec<GeoPoint>> = OnceCell::new();

/// R-tree leaf = index into `GEO_POINTS`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FeatureIdx(pub usize);

impl RTreeObject for FeatureIdx {
    type Envelope = AABB<[i32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let p = &GEO_POINTS.get().expect("geo table not initialised")[self.0];
        AABB::from_corners(
            [p.pb.longitude, p.pb.latitude],
            [p.pb.longitude, p.pb.latitude],
        )
    }
}

/// Haversine distance (caller already has radians).
#[inline]
pub fn fast_haversine(a: &GeoPoint, b: &GeoPoint) -> i32 {
    const R: f64 = 6_371_000.0; // Earth radius in metres
    let d_lat = b.lat_rad - a.lat_rad;
    let d_lon = b.lon_rad - a.lon_rad;

    let h = (d_lat * 0.5).sin().powi(2)
        + a.lat_rad.cos() * b.lat_rad.cos() * (d_lon * 0.5).sin().powi(2);

    (2.0 * R * h.sqrt().atan2((1.0 - h).sqrt())) as i32
}

// --------------------------------------------------------------------------
// Proto extension: make `Point` usable as a DashMap / HashMap key.
// --------------------------------------------------------------------------

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.latitude.hash(state);
        self.longitude.hash(state);
    }
}

impl Eq for Point {} // `PartialEq` is already derived by prost
