//! gRPC server entry-point
//!
//! After the library refactor, `RouteGuideService` is re-exported
//! from `tonic_tutorial::service`, and the generated proto module
//! is still available at `tonic_tutorial::proto`, so the old import
//! paths continue to work.

use std::net::SocketAddr;
use tonic::transport::Server;
use tonic_tutorial::{
    data, // helper: load route_guide_db.json
    proto::route_guide_server::RouteGuideServer,
    RouteGuideService, // re-exported in lib.rs
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ------------------------------------------------------------------
    // 1 · Load features once at start-up
    // ------------------------------------------------------------------
    let features = data::load()?; // Vec<proto::Feature>
    println!("✅ Loaded {} features", features.len());

    // ------------------------------------------------------------------
    // 2 · Instantiate the service (builds DashMap + R-tree internally)
    // ------------------------------------------------------------------
    let route_guide = RouteGuideService::new(features);

    // ------------------------------------------------------------------
    // 3 · Launch the gRPC server
    // ------------------------------------------------------------------
    let addr: SocketAddr = "[::1]:10000".parse()?;
    println!("🚀 gRPC server listening on http://{addr}");

    Server::builder()
        .add_service(RouteGuideServer::new(route_guide))
        .serve(addr)
        .await?;

    Ok(())
}
