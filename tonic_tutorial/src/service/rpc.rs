//! Only the gRPC trait implementation lives here.

use std::{pin::Pin, time::Instant};

use dashmap::DashMap;
use tokio_stream::{iter, Stream, StreamExt};
use tonic::{Request, Response, Status};

use crate::geometry::{fast_haversine, GeoPoint};
use crate::index::FeatureIndex;
use crate::proto::route_guide_server::RouteGuide;
use crate::proto::{Feature, Point, Rectangle, RouteNote, RouteSummary};

#[derive(Debug)]
pub struct RouteGuideService {
    idx: FeatureIndex,
}

impl RouteGuideService {
    pub fn new(features: Vec<Feature>) -> Self {
        Self {
            idx: FeatureIndex::build(features),
        }
    }
}

#[tonic::async_trait]
impl RouteGuide for RouteGuideService {
    // ------------------------------------------------------------------ get_feature
    async fn get_feature(&self, request: Request<Point>) -> Result<Response<Feature>, Status> {
        match self.idx.feature_map.get(request.get_ref()) {
            Some(f) => Ok(Response::new((**f).clone())),
            None => Ok(Response::new(Feature::default())),
        }
    }

    // ---------------------------------------------------------------- list_features
    type ListFeaturesStream = Pin<Box<dyn Stream<Item = Result<Feature, Status>> + Send + 'static>>;

    async fn list_features(
        &self,
        request: Request<Rectangle>,
    ) -> Result<Response<Self::ListFeaturesStream>, Status> {
        let rect = request.into_inner();
        let lo = rect.lo.as_ref().unwrap();
        let hi = rect.hi.as_ref().unwrap();
        let query =
            rstar::AABB::from_corners([lo.longitude, lo.latitude], [hi.longitude, hi.latitude]);

        // materialise hits so the stream owns its data
        let hits: Vec<Feature> = self
            .idx
            .rtree
            .locate_in_envelope(&query)
            .map(|idx| self.idx.features[idx.0].clone())
            .collect();

        Ok(Response::new(Box::pin(iter(hits.into_iter().map(Ok)))))
    }

    // ------------------------------------------------------------------ record_route
    async fn record_route(
        &self,
        request: Request<tonic::Streaming<Point>>,
    ) -> Result<Response<RouteSummary>, Status> {
        let mut stream = request.into_inner();
        let mut summary = RouteSummary::default();
        let mut last: Option<GeoPoint> = None;
        let start = Instant::now();

        while let Some(point) = stream.next().await {
            let point = point?;
            let geo = GeoPoint::from(point);
            summary.point_count += 1;

            if self.idx.feature_map.contains_key(&point) {
                summary.feature_count += 1;
            }
            if let Some(prev) = last {
                summary.distance = summary.distance.saturating_add(fast_haversine(&prev, &geo));
                // u32 → handles overflow gracefully
            }
            last = Some(geo);
        }

        summary.elapsed_time = start.elapsed().as_secs() as i32;
        Ok(Response::new(summary))
    }

    // -------------------------------------------------------------------- route_chat
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
