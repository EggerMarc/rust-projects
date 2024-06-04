#[derive(Debug)]
struct RouteGuideService {
    features: Arc<Vec<Feature>>,
}

pub mod proto {
    tonic::include_proto!("routeguide");
}

use proto::route_guide_server::{RouteGuide, RouteGuideServer};
use proto::{Feature, Point, Rectangle, RouteNote, RouteSummary};

use std::pin::Pin; //Pinned pointer
use std::sync::Arc; // Atomically referenced counter ?
use std::cmp; //utilities for comparing and ordering
use tokio::sync::mpsc; // Multi-producer, single consumer queue for sending values between
                       // async tasks
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{Request, Response, Status};


use std::hash::{Hash, Hasher};
impl Hash for Point {
    fn hash<H>(&self, state: &mut H)
    where H: Hasher, {
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

    point.longitude >= left && point.longitude <= right && point.latitude >= down && point.latitude <= top
}

fn calc_distance(p1: &Point, p2: &Point) -> i32 {
    // Tutorial uses haversine distance
    let cord_factor: f64 = 1e7;   
    let r: f64 = 6_371_000.0;

    let delta_lo = ((p1.longitude - p2.longitude) as f64 / cord_factor).to_radians();
    let delta_la = ((p1.latitude - p1.latitude) as f64 / cord_factor).to_radians();
    let lat_ra1 = (p1.latitude as f64).to_radians();
    let lat_ra2 = (p2.latitude as f64).to_radians();
    
    let a = ( delta_la / 2f64).sin() * (delta_la / 2f64).sin() +
        (lat_ra1).cos() * (lat_ra2).cos() * (delta_lo / 2f64).sin() ** 2; 


}

#[derive(Debug)]
struct RouteGuideService {
    features: Arc<Vec<Feature>>,
}

pub mod proto {
    tonic::include_proto!("routeguide");
}

use proto::route_guide_server::{RouteGuide, RouteGuideServer};
use proto::{Feature, Point, Rectangle, RouteNote, RouteSummary};

use std::pin::Pin; //Pinned pointer
use std::sync::Arc; // Atomically referenced counter ?
use std::cmp; //utilities for comparing and ordering
use tokio::sync::mpsc; // Multi-producer, single consumer queue for sending values between
                       // async tasks
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{Request, Response, Status};


use std::hash::{Hash, Hasher};
impl Hash for Point {
    fn hash<H>(&self, state: &mut H)
    where H: Hasher, {
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

    point.longitude >= left && point.longitude <= right && point.latitude >= down && point.latitude <= top


#[derive(Debug)]
struct RouteGuideService {
    features: Arc<Vec<Feature>>,
}

pub mod proto {
    tonic::include_proto!("routeguide");
}

use proto::route_guide_server::{RouteGuide, RouteGuideServer};
use proto::{Feature, Point, Rectangle, RouteNote, RouteSummary};

use std::pin::Pin; //Pinned pointer
use std::sync::Arc; // Atomically referenced counter ?
use std::cmp; //utilities for comparing and ordering
use tokio::sync::mpsc; // Multi-producer, single consumer queue for sending values between
                       // async tasks
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{Request, Response, Status};


use std::hash::{Hash, Hasher};
impl Hash for Point {
    fn hash<H>(&self, state: &mut H)
    where H: Hasher, {
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

    point.longitude >= left && point.longitude <= right && point.latitude >= down && point.latitude <= top
}

#[tonic::async_trait]
impl RouteGuide for RouteGuideService {
    async fn get_feature(&self, _request: Request<Point>) -> Result<Response<Feature>, Status> {
        unimplemented!()
    }

    // Response streams have to be typed out?
    type ListFeaturesStream = ReceiverStream<Result<Feature, Status>>;

    async fn list_features(
        &self,
        _request: Request<Rectangle>,
    ) -> Result<Response<Self::ListFeaturesStream>, Status> {
        unimplemented!()
    }

    async fn record_route(
        &self,
        _request: Request<tonic::Streaming<Point>>,
    ) -> Result<Response<RouteSummary>, Status> {
        unimplemented!()
    }

    // HELP
    type RouteChatStream = Pin<Box<dyn Stream<Item = Result<RouteNote, Status>> + Send + 'static>>;

    async fn route_chat(
        &self,
        _request: Request<tonic::Streaming<Point>>,
    ) -> Result<Response<Self::RouteChatStream>, Status> {
        unimplemented!()
    }
}

fn main() {
    println!("Hey mom")
}
}

#[tonic::async_trait]
impl RouteGuide for RouteGuideService {
    async fn get_feature(&self, _request: Request<Point>) -> Result<Response<Feature>, Status> {
        unimplemented!()
    }

    // Response streams have to be typed out?
    type ListFeaturesStream = ReceiverStream<Result<Feature, Status>>;

    async fn list_features(
        &self,
        _request: Request<Rectangle>,
    ) -> Result<Response<Self::ListFeaturesStream>, Status> {
        unimplemented!()
    }

    async fn record_route(
        &self,
        _request: Request<tonic::Streaming<Point>>,
    ) -> Result<Response<RouteSummary>, Status> {
        unimplemented!()
    }

    // HELP
    type RouteChatStream = Pin<Box<dyn Stream<Item = Result<RouteNote, Status>> + Send + 'static>>;

    async fn route_chat(
        &self,
        _request: Request<tonic::Streaming<Point>>,
    ) -> Result<Response<Self::RouteChatStream>, Status> {
        unimplemented!()
    }
}

fn main() {
    println!("Hey mom")
}

#[derive(Debug)]
struct RouteGuideService {
    features: Arc<Vec<Feature>>,
}

pub mod proto {
    tonic::include_proto!("routeguide");
}

use proto::route_guide_server::{RouteGuide, RouteGuideServer};
use proto::{Feature, Point, Rectangle, RouteNote, RouteSummary};

use std::pin::Pin; //Pinned pointer
use std::sync::Arc; // Atomically referenced counter ?
use std::cmp; //utilities for comparing and ordering
use tokio::sync::mpsc; // Multi-producer, single consumer queue for sending values between
                       // async tasks
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{Request, Response, Status};


use std::hash::{Hash, Hasher};
impl Hash for Point {
    fn hash<H>(&self, state: &mut H)
    where H: Hasher, {
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

    point.longitude >= left && point.longitude <= right && point.latitude >= down && point.latitude <= top
}

#[tonic::async_trait]
impl RouteGuide for RouteGuideService {
    async fn get_feature(&self, _request: Request<Point>) -> Result<Response<Feature>, Status> {
        unimplemented!()
    }

    // Response streams have to be typed out?
    type ListFeaturesStream = ReceiverStream<Result<Feature, Status>>;

    async fn list_features(
        &self,
        _request: Request<Rectangle>,
    ) -> Result<Response<Self::ListFeaturesStream>, Status> {
        unimplemented!()
    }

    async fn record_route(
        &self,
        _request: Request<tonic::Streaming<Point>>,
    ) -> Result<Response<RouteSummary>, Status> {
        unimplemented!()
    }

    // HELP
    type RouteChatStream = Pin<Box<dyn Stream<Item = Result<RouteNote, Status>> + Send + 'static>>;

    async fn route_chat(
        &self,
        _request: Request<tonic::Streaming<Point>>,
    ) -> Result<Response<Self::RouteChatStream>, Status> {
        unimplemented!()
    }
}

fn main() {
    println!("Hey mom")
}
}

#[tonic::async_trait]
impl RouteGuide for RouteGuideService {
    async fn get_feature(&self, _request: Request<Point>) -> Result<Response<Feature>, Status> {
        unimplemented!()
    }

    // Response streams have to be typed out?
    type ListFeaturesStream = ReceiverStream<Result<Feature, Status>>;

    async fn list_features(
        &self,
        _request: Request<Rectangle>,
    ) -> Result<Response<Self::ListFeaturesStream>, Status> {
        unimplemented!()
    }

    async fn record_route(
        &self,
        _request: Request<tonic::Streaming<Point>>,
    ) -> Result<Response<RouteSummary>, Status> {
        unimplemented!()
    }

    // HELP
    type RouteChatStream = Pin<Box<dyn Stream<Item = Result<RouteNote, Status>> + Send + 'static>>;

    async fn route_chat(
        &self,
        _request: Request<tonic::Streaming<Point>>,
    ) -> Result<Response<Self::RouteChatStream>, Status> {
        unimplemented!()
    }
}

fn main() {
    println!("Hey mom")
}
