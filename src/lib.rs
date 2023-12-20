//! RBC Aggregation Analysis
//!
//! A toolkit for analyzing red blood cell aggregation from 3D confocal microscopy data.

pub mod mesh;
pub mod io;

pub use mesh::{Mesh, InertiaProperties};
pub use io::ObjLoader;
