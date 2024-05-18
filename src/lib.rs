//! RBC Aggregation Analysis
//!
//! A high-performance toolkit for analyzing red blood cell aggregation
//! from 3D confocal microscopy data. Implements mesh processing, inertia
//! tensor computation, and spatial correlation functions.

pub mod mesh;
pub mod analysis;
pub mod io;

pub use mesh::{Mesh, MeshFilter, InertiaProperties};
pub use analysis::{PairCorrelation, VolumeDistribution};
pub use io::{ObjLoader, TiffStack};
