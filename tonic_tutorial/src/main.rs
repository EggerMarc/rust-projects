use std::net::SocketAddr;
use tonic::transport::Server;
use tonic_tutorial::{
    data, // helper that deserialises route_guide_db.json
    proto::route_guide_server::RouteGuideServer,
    RouteGuideService,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ------------------------------------------------------------------
    // 1 · Load the features once at start-up
    // ------------------------------------------------------------------
    let features = data::load()?; // Vec<proto::Feature>
    println!("✅ Loaded {} features", features.len());

    // ------------------------------------------------------------------
    // 2 · Instantiate the service (builds maps + R-tree inside)
    // ------------------------------------------------------------------
    let route_guide = RouteGuideService::new(features);

    // ------------------------------------------------------------------
    // 3 · Launch the gRPC server
    // ------------------------------------------------------------------
    let addr: SocketAddr = "[::1]:10000".parse()?;
    println!("🚀 Starting gRPC server on http://{}", addr);

    Server::builder()
        .add_service(RouteGuideServer::new(route_guide))
        .serve(addr)
        .await?;

    Ok(())
}
