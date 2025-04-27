//! tests/route_guide.rs
//! Integration tests for RouteGuideService

use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tokio_stream::StreamExt;
use tonic::transport::Server;
use tonic_tutorial::{
    data,
    proto::{
        route_guide_client::RouteGuideClient, route_guide_server::RouteGuideServer, Point,
        Rectangle, RouteNote,
    },
    service::RouteGuideService,
};

// --------------------------------------------------------------------- helpers

const SERVER_STARTUP_DELAY_MS: u64 = 100;

async fn start_test_server(port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // Load the data set and hand it straight to the constructor
    let features = data::load().expect("failed to load features");
    let route_guide = RouteGuideService::new(features);

    tokio::spawn(async move {
        println!("🚀 Starting test server on {}", addr);
        Server::builder()
            .add_service(RouteGuideServer::new(route_guide))
            .serve(addr)
            .await
            .expect("server crashed");
    });

    // Give the async server a moment to bind the socket
    sleep(Duration::from_millis(SERVER_STARTUP_DELAY_MS)).await;
}

async fn setup_client(port: u16) -> RouteGuideClient<tonic::transport::Channel> {
    RouteGuideClient::connect(format!("http://127.0.0.1:{port}"))
        .await
        .expect("client connect failed")
}

// --------------------------------------------------------------------- tests

#[tokio::test]
async fn get_feature_returns_expected_feature() {
    const PORT: u16 = 50_051;
    start_test_server(PORT).await;
    let mut client = setup_client(PORT).await;

    let response = client
        .get_feature(tonic::Request::new(Point {
            latitude: 409_146_138,
            longitude: -746_188_906,
        }))
        .await
        .expect("get_feature failed");

    assert_eq!(
        response.into_inner().name,
        "Berkshire Valley Management Area Trail, Jefferson, NJ, USA"
    );
}

#[tokio::test]
async fn list_features_returns_features_in_rectangle() {
    const PORT: u16 = 50_052;
    start_test_server(PORT).await;
    let mut client = setup_client(PORT).await;

    let mut stream = client
        .list_features(tonic::Request::new(Rectangle {
            lo: Some(Point {
                latitude: 400_000_000,
                longitude: -750_000_000,
            }),
            hi: Some(Point {
                latitude: 420_000_000,
                longitude: -730_000_000,
            }),
        }))
        .await
        .expect("list_features failed")
        .into_inner();

    let mut count = 0;
    while let Some(Ok(feature)) = stream.next().await {
        assert!(feature.location.is_some());
        count += 1;
    }

    assert!(count > 0, "expected at least one feature in rectangle");
}

#[tokio::test]
async fn record_route_returns_summary() {
    const PORT: u16 = 50_053;
    start_test_server(PORT).await;
    let mut client = setup_client(PORT).await;

    let points = vec![
        Point {
            latitude: 409_146_138,
            longitude: -746_188_906,
        },
        Point {
            latitude: 410_248_224,
            longitude: -743_099_979,
        },
    ];

    let summary = client
        .record_route(tonic::Request::new(tokio_stream::iter(points)))
        .await
        .expect("record_route failed")
        .into_inner();

    assert_eq!(summary.point_count, 2);
    assert!(summary.distance > 0);
    assert!(summary.elapsed_time >= 0);
}

#[tokio::test]
async fn route_chat_exchanges_notes() {
    const PORT: u16 = 50_054;
    start_test_server(PORT).await;
    let mut client = setup_client(PORT).await;

    let notes = vec![
        RouteNote {
            location: Some(Point {
                latitude: 409_146_138,
                longitude: -746_188_906,
            }),
            message: "First note".into(),
        },
        RouteNote {
            location: Some(Point {
                latitude: 409_146_138,
                longitude: -746_188_906,
            }),
            message: "Second note".into(),
        },
    ];

    let mut stream = client
        .route_chat(tonic::Request::new(tokio_stream::iter(notes)))
        .await
        .expect("route_chat failed")
        .into_inner();

    let mut received = Vec::new();
    while let Some(note) = stream.message().await.unwrap() {
        received.push(note.message);
    }

    assert!(received.contains(&"First note".to_string()));
    assert!(received.contains(&"Second note".to_string()));
}
