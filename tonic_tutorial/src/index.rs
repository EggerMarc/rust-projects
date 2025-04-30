//! Build-time immutable lookup structures shared by all RPC handlers.

use std::sync::Arc;

use dashmap::DashMap;
use rstar::RTree;

use crate::geometry::{FeatureIdx, GeoPoint, GEO_POINTS};
use crate::proto::{Feature, Point};

#[derive(Debug)]
pub struct FeatureIndex {
    pub features: Arc<[Feature]>,
    pub feature_map: DashMap<Point, Arc<Feature>>,
    pub rtree: RTree<FeatureIdx>,
}

impl FeatureIndex {
    /// Builds the exact-match map, the global `GeoPoint` table and the R-tree.
    pub fn build(features: Vec<Feature>) -> Self {
        // 1 ▸ Move vector into a ref-counted, immutable slice.
        let features = Arc::<[Feature]>::from(features);

        // 2 ▸ Pre-allocate helper structures.
        let feature_map = DashMap::with_capacity(features.len());
        let mut geo_points = Vec::with_capacity(features.len());
        let mut indices = Vec::with_capacity(features.len());

        // 3 ▸ Populate helpers.
        for (idx, feat) in features.iter().enumerate() {
            if let Some(loc) = &feat.location {
                // `loc` is `&Point`; `Point` is `Copy`, so just deref.
                let key: Point = *loc;

                // 3a exact-match table
                feature_map.insert(key, Arc::new(feat.clone()));

                // 3b coordinate table for R-tree
                geo_points.push(GeoPoint::from(key));
                indices.push(FeatureIdx(idx));
            }
        }

        // 4 ▸ Publish coordinate table exactly once.
        if GEO_POINTS.get().is_none() {
            let _ = GEO_POINTS.set(geo_points);
        }

        // 5 ▸ Bulk-load R-tree.
        Self {
            features,
            feature_map,
            rtree: RTree::bulk_load(indices),
        }
    }
}
