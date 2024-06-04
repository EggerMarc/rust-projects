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
use tokio::sync::mpsc; // Multi-producer, single consumer queue for sending values between
                       // async tasks
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{Request, Response, Status};

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
