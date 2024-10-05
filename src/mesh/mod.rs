//! Mesh processing module for 3D cell representations.

mod geometry;
pub use geometry::Mesh;
mod inertia;
pub use inertia::InertiaProperties;
mod filter;
pub use filter::MeshFilter;
