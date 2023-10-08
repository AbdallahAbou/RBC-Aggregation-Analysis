//! Core mesh geometry representation and operations.

use nalgebra::{Point3, Vector3};

/// Triangular mesh representation of a 3D cell.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Vertex positions
    pub vertices: Vec<Point3<f64>>,
    /// Triangle indices (i, j, k) referencing vertices
    pub faces: Vec<[usize; 3]>,
    /// Source file identifier
    pub id: String,
}

impl Mesh {
    /// Create a new mesh from vertices and faces.
    pub fn new(vertices: Vec<Point3<f64>>, faces: Vec<[usize; 3]>, id: String) -> Self {
        Self { vertices, faces, id }
    }
}
