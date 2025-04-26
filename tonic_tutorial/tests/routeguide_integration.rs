use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio_stream::StreamExt;
use tonic::transport::Server;
use tonic_tutorial::data;
use tonic_tutorial::proto::route_guide_client::RouteGuideClient;
use tonic_tutorial::proto::route_guide_server::RouteGuideServer;
use tonic_tutorial::proto::{Feature, Point, Rectangle, RouteNote};
use tonic_tutorial::service::RouteGuideService;

// ==== Helpers ====

const SERVER_STARTUP_DELAY_MS: u64 = 100;

/// Inicia o servidor gRPC na porta especificada
async fn start_test_server(port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let features_vec = data::load().expect("Failed to load features");
    let features: Arc<[Feature]> = Arc::from(features_vec.into_boxed_slice());

    let feature_map: HashMap<Point, Feature> = features
        .iter()
        .filter_map(|f| f.location.map(|loc| (loc, f.clone())))
        .collect();

    let route_guide = RouteGuideService {
        features,
        feature_map,
    };

    tokio::spawn(async move {
        let svc = RouteGuideServer::new(route_guide);
        println!("🚀 Starting test server on {}", addr);
        Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .expect("Server failed");
    });

    sleep(Duration::from_millis(SERVER_STARTUP_DELAY_MS)).await;
}

/// Cria um client gRPC para a porta especificada
async fn setup_client(port: u16) -> RouteGuideClient<tonic::transport::Channel> {
    RouteGuideClient::connect(format!("http://127.0.0.1:{}", port))
        .await
        .expect("client connect failed")
}

// ==== Tests ====

#[tokio::test]
async fn test_get_feature_returns_expected_feature() {
    const PORT: u16 = 50051;
    start_test_server(PORT).await;
    let mut client = setup_client(PORT).await;

    let request = tonic::Request::new(Point {
        latitude: 409146138,
        longitude: -746188906,
    });

    let response = client
        .get_feature(request)
        .await
        .expect("get_feature failed");

    let feature: Feature = response.into_inner();
    assert_eq!(
        feature.name,
        "Berkshire Valley Management Area Trail, Jefferson, NJ, USA"
    );
}

#[tokio::test]
async fn test_list_features_returns_features_in_rectangle() {
    const PORT: u16 = 50052;
    start_test_server(PORT).await;
    let mut client = setup_client(PORT).await;

    let rectangle = tonic::Request::new(Rectangle {
        lo: Some(Point {
            latitude: 400000000,
            longitude: -750000000,
        }),
        hi: Some(Point {
            latitude: 420000000,
            longitude: -730000000,
        }),
    });

    let mut stream = client
        .list_features(rectangle)
        .await
        .expect("list_features failed")
        .into_inner();

    let mut count = 0;
    while let Some(Ok(feature)) = stream.next().await {
        assert!(feature.location.is_some());
        count += 1;
    }

    assert!(count > 0, "Expected at least one feature in rectangle");
}

#[tokio::test]
async fn test_record_route_returns_summary() {
    const PORT: u16 = 50053;
    start_test_server(PORT).await;
    let mut client = setup_client(PORT).await;

    let points = vec![
        Point {
            latitude: 409146138,
            longitude: -746188906,
        },
        Point {
            latitude: 410248224,
            longitude: -743099979,
        },
    ];

    let stream = tokio_stream::iter(points);
    let request = tonic::Request::new(stream);

    let response = client
        .record_route(request)
        .await
        .expect("record_route failed");

    let summary = response.into_inner();
    assert_eq!(summary.point_count, 2);
    assert!(summary.distance > 0);
    assert!(summary.elapsed_time >= 0);
}

#[tokio::test]
async fn test_route_chat_exchanges_notes() {
    const PORT: u16 = 50054;
    start_test_server(PORT).await;
    let mut client = setup_client(PORT).await;

    let notes = vec![
        RouteNote {
            location: Some(Point {
                latitude: 409146138,
                longitude: -746188906,
            }),
            message: "First note".to_string(),
        },
        RouteNote {
            location: Some(Point {
                latitude: 409146138,
                longitude: -746188906,
            }),
            message: "Second note".to_string(),
        },
    ];

    let stream = tokio_stream::iter(notes);
    let request = tonic::Request::new(stream);

    let mut response_stream = client
        .route_chat(request)
        .await
        .expect("route_chat failed")
        .into_inner();

    let mut received_messages = Vec::new();
    while let Some(note) = response_stream.message().await.unwrap() {
        received_messages.push(note.message);
    }

    assert!(received_messages.contains(&"First note".to_string()));
    assert!(received_messages.contains(&"Second note".to_string()));
}
