//! Library root — re-exports public API and wires internal modules.

pub mod data;
pub mod geometry;
mod index;
pub mod service;

pub use service::RouteGuideService;

// prost-generated code lives here; adjust the path to match your build.rs
pub mod proto {
    tonic::include_proto!("routeguide");
}
