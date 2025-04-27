use crate::proto::route_guide_server::RouteGuide;
use crate::proto::{Feature, Point, Rectangle, RouteNote, RouteSummary};

use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rstar::{RTree, RTreeObject, AABB};
use std::{pin::Pin, sync::Arc, time::Instant};
use tokio_stream::{iter, Stream, StreamExt};
use tonic::{Request, Response, Status};

use std::hash::{Hash, Hasher};

/// ---- Geometry helpers ----------------------------------------------------

#[derive(Debug, Clone)]
struct GeoPoint {
    pb: Point, // lat / lon stored in 1 e-7 °
    lat_rad: f64,
    lon_rad: f64,
}

impl From<Point> for GeoPoint {
    fn from(pb: Point) -> Self {
        let scale = 1e7_f64;
        Self {
            lat_rad: (pb.latitude as f64 / scale).to_radians(),
            lon_rad: (pb.longitude as f64 / scale).to_radians(),
            pb,
        }
    }
}

/// Pre-initialised, immutable table that every R-tree leaf points into.
static GEO_POINTS: OnceCell<Vec<GeoPoint>> = OnceCell::new();

// ---- make prost::Point hash-able and Eq so we can use it as a DashMap key ----
impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.latitude.hash(state);
        self.longitude.hash(state);
    }
}

impl Eq for Point {} // PartialEq is already derived by prost

/// Leaf = just an index into `GEO_POINTS`.
#[derive(Debug, Clone, Copy)]
struct FeatureIdx(usize);

impl RTreeObject for FeatureIdx {
    type Envelope = AABB<[i32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let p = &GEO_POINTS.get().expect("geo table")[self.0];
        AABB::from_corners(
            [p.pb.longitude, p.pb.latitude],
            [p.pb.longitude, p.pb.latitude],
        )
    }
}

/// ---- gRPC service --------------------------------------------------------

#[derive(Debug)]
pub struct RouteGuideService {
    features: Arc<[Feature]>,
    feature_map: DashMap<Point, Arc<Feature>>,
    rtree: RTree<FeatureIdx>,
}

impl RouteGuideService {
    pub fn new(features: Vec<Feature>) -> Self {
        // 1. Move the Vec into an Arc slice
        let features = Arc::<[Feature]>::from(features);

        // 2. Build look-up structures
        let feature_map = DashMap::with_capacity(features.len());
        let mut geo_points = Vec::with_capacity(features.len());
        let mut indices = Vec::with_capacity(features.len());

        for (idx, feat) in features.iter().enumerate() {
            if let Some(loc) = &feat.location {
                // store an Arc for O(1) exact look-ups
                feature_map.insert(loc.clone(), Arc::new(feat.clone()));

                // prepare spatial index
                geo_points.push(GeoPoint::from(loc.clone()));
                indices.push(FeatureIdx(idx));
            }
        }

        // 3. Publish the coordinate table exactly once
        let _ = GEO_POINTS.set(geo_points);

        Self {
            features,
            feature_map,
            rtree: RTree::bulk_load(indices),
        }
    }
}

#[tonic::async_trait]
impl RouteGuide for RouteGuideService {
    async fn get_feature(&self, request: Request<Point>) -> Result<Response<Feature>, Status> {
        match self.feature_map.get(request.get_ref()) {
            Some(f) => Ok(Response::new((**f).clone())),
            None => Ok(Response::new(Feature::default())),
        }
    }

    type ListFeaturesStream = Pin<Box<dyn Stream<Item = Result<Feature, Status>> + Send + 'static>>;

    async fn list_features(
        &self,
        request: Request<Rectangle>,
    ) -> Result<Response<Self::ListFeaturesStream>, Status> {
        let rect = request.into_inner();
        let lo = rect.lo.as_ref().unwrap();
        let hi = rect.hi.as_ref().unwrap();
        let query = AABB::from_corners([lo.longitude, lo.latitude], [hi.longitude, hi.latitude]);

        // Collect matching features into an owned Vec so the stream owns its data
        let hits: Vec<Feature> = self
            .rtree
            .locate_in_envelope(&query)
            .map(|idx| self.features[idx.0].clone())
            .collect();

        let output = iter(hits.into_iter().map(Ok));
        Ok(Response::new(Box::pin(output)))
    }

    async fn record_route(
        &self,
        request: Request<tonic::Streaming<Point>>,
    ) -> Result<Response<RouteSummary>, Status> {
        let mut stream = request.into_inner();
        let mut summary = RouteSummary::default();
        let mut last_point: Option<GeoPoint> = None;
        let start = Instant::now();

        while let Some(point) = stream.next().await {
            let point = point?;
            let geo = GeoPoint::from(point);
            summary.point_count += 1;

            if self.feature_map.contains_key(&point) {
                summary.feature_count += 1;
            }
            if let Some(prev) = &last_point {
                summary.distance += fast_haversine(prev, &geo);
            }
            last_point = Some(geo);
        }

        summary.elapsed_time = start.elapsed().as_secs() as i32;
        Ok(Response::new(summary))
    }

    type RouteChatStream = Pin<Box<dyn Stream<Item = Result<RouteNote, Status>> + Send + 'static>>;

    async fn route_chat(
        &self,
        request: Request<tonic::Streaming<RouteNote>>,
    ) -> Result<Response<Self::RouteChatStream>, Status> {
        let notes: DashMap<Point, Vec<RouteNote>> = DashMap::new();
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            while let Some(note) = stream.next().await {
                let note = note?;
                let loc  = note.location.unwrap();
                let mut entry = notes.entry(loc).or_default();
                entry.push(note.clone());
                for n in entry.iter() {
                    yield n.clone();
                }
            }
        };

        Ok(Response::new(Box::pin(output)))
    }
}

/// ---- Fast haversine (caller already has radians) -------------------------
#[inline]
fn fast_haversine(a: &GeoPoint, b: &GeoPoint) -> i32 {
    const R: f64 = 6_371_000.0;
    let d_lat = b.lat_rad - a.lat_rad;
    let d_lon = b.lon_rad - a.lon_rad;

    let h = (d_lat * 0.5).sin().powi(2)
        + a.lat_rad.cos() * b.lat_rad.cos() * (d_lon * 0.5).sin().powi(2);

    (2.0 * R * h.sqrt().atan2((1.0 - h).sqrt())) as i32
}
