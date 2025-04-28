pub mod proto {
    tonic::include_proto!("routeguide");
}

mod data;
use tonic::transport::Server;

use proto::route_guide_server::{RouteGuide, RouteGuideServer};
use proto::{Feature, Point, Rectangle, RouteNote, RouteSummary};

use std::cmp; //utilities for comparing and ordering
use std::pin::Pin; //Pinned pointer
use std::sync::Arc; // Atomically referenced counter ?
use std::time::Instant;

use tokio::sync::mpsc; // Multi-producer, single consumer queue for sending values between
                       // async tasks
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Request, Response, Status};

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

impl Hash for Point {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.latitude.hash(state);
        self.longitude.hash(state);
    }
}

impl Eq for Point {}

fn in_range(point: &Point, rectangle: &Rectangle) -> bool {
    let _lo = rectangle.lo.as_ref().unwrap();
    let _hi = rectangle.hi.as_ref().unwrap();

    let top = cmp::max(_lo.latitude, _hi.latitude);
    let down = cmp::min(_lo.latitude, _hi.latitude);
    let left = cmp::min(_lo.longitude, _lo.longitude);
    let right = cmp::max(_lo.longitude, _lo.longitude);

    point.longitude >= left
        && point.longitude <= right
        && point.latitude >= down
        && point.latitude <= top
}

fn calc_distance(p1: &Point, p2: &Point) -> i32 {
    // Tutorial uses haversine distance
    let cord_factor: f64 = 1e7;
    let r: f64 = 6_371_000.0;

    let delta_lo = ((p1.longitude - p2.longitude) as f64 / cord_factor).to_radians();
    let delta_la = ((p1.latitude - p1.latitude) as f64 / cord_factor).to_radians();
    let lat_ra1 = (p1.latitude as f64).to_radians();
    let lat_ra2 = (p2.latitude as f64).to_radians();

    let a = (delta_la / 2f64).sin() * (delta_la / 2f64).sin()
        + (lat_ra1).cos() * (lat_ra2).cos() * (delta_lo / 2f64).sin().powf(2.0);

    let c = 2f64 * a.sqrt().atan2((1f64 - a).sqrt());

    (r * c) as i32
}

#[derive(Debug)]
struct RouteGuideService {
    features: Arc<Vec<Feature>>,
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

    // Response streams have to be typed out?
    type ListFeaturesStream = ReceiverStream<Result<Feature, Status>>;

    async fn list_features(
        &self,
        request: Request<Rectangle>,
    ) -> Result<Response<Self::ListFeaturesStream>, Status> {
        // Create a channel to return stream of values
        let (mut tx, rx) = mpsc::channel(4);
        let features = self.features.clone();

        tokio::spawn(async move {
            for feature in &features[..] {
                if in_range(feature.location.as_ref().unwrap(), request.get_ref()) {
                    tx.send(Ok(feature.clone())).await.unwrap();
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

        // loop over stream and "match" points
        while let Some(point) = stream.next().await {
            let point = point?;
            summary.point_count += 1;

            // Usual point feature "getter"
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

    // HELP
    type RouteChatStream = Pin<Box<dyn Stream<Item = Result<RouteNote, Status>> + Send + 'static>>;

    async fn route_chat(
        &self,
        request: Request<tonic::Streaming<RouteNote>>,
    ) -> Result<Response<Self::RouteChatStream>, Status> {
        let mut notes = HashMap::new();
        let mut stream = request.into_inner();

        // perform async transformations between input & output streams
        let output = async_stream::try_stream! {
            while let Some(note) = stream.next().await {
                let note = note?;
                let location = note.location.clone().unwrap();
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse().unwrap(); // Corrigi também aqui! Estava faltando o ":"
    
    let route_guide = RouteGuideService {
        features: Arc::new(data::load()?), // <- agora usando ?
    };

    let svc = RouteGuideServer::new(route_guide);
    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}
