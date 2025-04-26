use std::sync::Arc;
use tonic::transport::Server;
use tonic_tutorial::proto::route_guide_server::RouteGuideServer;
use tonic_tutorial::{data, RouteGuideService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse()?;

    let features = data::load()?;
    println!("✅ Loaded {} features", features.len());

    let route_guide = RouteGuideService {
        features: Arc::new(features),
    };

    println!("🚀 Starting gRPC server on http://{}", addr);

    let svc = RouteGuideServer::new(route_guide);
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
