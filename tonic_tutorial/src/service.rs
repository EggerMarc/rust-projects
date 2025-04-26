use crate::proto::route_guide_server::RouteGuide;
use crate::proto::{Feature, Point, Rectangle, RouteNote, RouteSummary};
use std::cmp;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Request, Response, Status};

// Implementar Hash para Point
impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.latitude.hash(state);
        self.longitude.hash(state);
    }
}

impl Eq for Point {}

#[derive(Debug)]
pub struct RouteGuideService {
    pub features: Arc<Vec<Feature>>,
}

#[tonic::async_trait]
impl RouteGuide for RouteGuideService {
    async fn get_feature(&self, request: Request<Point>) -> Result<Response<Feature>, Status> {
        for feature in &self.features[..] {
            if feature.location.as_ref() == Some(request.get_ref()) {
                return Ok(Response::new(feature.clone()));
            }
        }
        Ok(Response::new(Feature::default()))
    }

    type ListFeaturesStream = ReceiverStream<Result<Feature, Status>>;

    async fn list_features(
        &self,
        request: Request<Rectangle>,
    ) -> Result<Response<Self::ListFeaturesStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let features = self.features.clone();

        tokio::spawn(async move {
            for feature in &features[..] {
                if let Some(location) = &feature.location {
                    if in_range(location, request.get_ref())
                        && tx.send(Ok(feature.clone())).await.is_err()
                    {
                        break;
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn record_route(
        &self,
        request: Request<tonic::Streaming<Point>>,
    ) -> Result<Response<RouteSummary>, Status> {
        let mut stream = request.into_inner();
        let mut summary = RouteSummary::default();
        let mut last_point = None;
        let now = Instant::now();

        while let Some(point) = stream.next().await {
            let point = point?;
            summary.point_count += 1;

            for feature in &self.features[..] {
                if feature.location.as_ref() == Some(&point) {
                    summary.feature_count += 1;
                }
            }

            if let Some(ref last_point) = last_point {
                summary.distance += calc_distance(last_point, &point);
            }

            last_point = Some(point);
        }

        summary.elapsed_time = now.elapsed().as_secs() as i32;
        Ok(Response::new(summary))
    }

    type RouteChatStream = Pin<Box<dyn Stream<Item = Result<RouteNote, Status>> + Send + 'static>>;

    async fn route_chat(
        &self,
        request: Request<tonic::Streaming<RouteNote>>,
    ) -> Result<Response<Self::RouteChatStream>, Status> {
        let mut notes = HashMap::new();
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            while let Some(note) = stream.next().await {
                let note = note?;
                let location = note.location.unwrap();
                let location_notes = notes.entry(location).or_insert(vec![]);
                location_notes.push(note);

                for note in location_notes {
                    yield note.clone();
                }
            }
        };

        Ok(Response::new(Box::pin(output)))
    }
}

// Funções auxiliares
fn in_range(point: &Point, rectangle: &Rectangle) -> bool {
    let lo = rectangle.lo.as_ref().unwrap();
    let hi = rectangle.hi.as_ref().unwrap();

    let top = cmp::max(lo.latitude, hi.latitude);
    let down = cmp::min(lo.latitude, hi.latitude);
    let left = cmp::min(lo.longitude, hi.longitude);
    let right = cmp::max(lo.longitude, hi.longitude);

    point.longitude >= left
        && point.longitude <= right
        && point.latitude >= down
        && point.latitude <= top
}

fn calc_distance(p1: &Point, p2: &Point) -> i32 {
    let cord_factor: f64 = 1e7;
    let r: f64 = 6_371_000.0;

    let delta_lo = ((p1.longitude - p2.longitude) as f64 / cord_factor).to_radians();
    let delta_la = ((p1.latitude - p2.latitude) as f64 / cord_factor).to_radians();
    let lat_ra1 = (p1.latitude as f64 / cord_factor).to_radians();
    let lat_ra2 = (p2.latitude as f64 / cord_factor).to_radians();

    let a = (delta_la / 2f64).sin().powi(2)
        + (lat_ra1).cos() * (lat_ra2).cos() * (delta_lo / 2f64).sin().powi(2);

    let c = 2f64 * a.sqrt().atan2((1f64 - a).sqrt());

    (r * c) as i32
}
