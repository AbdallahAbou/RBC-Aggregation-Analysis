//! Mesh processing and geometric analysis for 3D cell representations.

mod geometry;
mod filter;
mod inertia;

pub use geometry::Mesh;
pub use filter::MeshFilter;
pub use inertia::InertiaProperties;

use serde::{Deserialize, Serialize};

/// Record containing extracted cell properties for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellRecord {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub lambda1: f64,
    pub lambda2: f64,
    pub lambda3: f64,
    pub asphericity: f64,
    pub acylindricity: f64,
    pub volume: f64,
}
