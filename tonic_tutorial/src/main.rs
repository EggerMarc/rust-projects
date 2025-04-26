use std::sync::Arc;
use tonic::transport::Server;
use tonic_tutorial::proto::route_guide_server::RouteGuideServer;
use tonic_tutorial::{data, RouteGuideService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse()?;

    let features_vec = data::load()?;
    println!("✅ Loaded {} features", features_vec.len());

    let features: Arc<[tonic_tutorial::proto::Feature]> =
        Arc::from(features_vec.into_boxed_slice());

    let feature_map = features
        .iter()
        .filter_map(|f| f.location.map(|loc| (loc, f.clone())))
        .collect();

    let route_guide = RouteGuideService {
        features,
        feature_map,
    };

    println!("🚀 Starting gRPC server on http://{}", addr);

    let svc = RouteGuideServer::new(route_guide);
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
