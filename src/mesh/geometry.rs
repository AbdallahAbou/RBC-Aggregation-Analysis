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

impl Mesh {
    /// Signed volume of mesh using divergence theorem.
    pub fn compute_volume(&self) -> f64 {
        let mut volume = 0.0;

        for face in &self.faces {
            let v0 = &self.vertices[face[0]];
            let v1 = &self.vertices[face[1]];
            let v2 = &self.vertices[face[2]];

            // Signed volume of tetrahedron formed with origin
            volume += v0.coords.dot(&v1.coords.cross(&v2.coords));
        }

        (volume / 6.0).abs()
    }
}

impl Mesh {
    /// Compute centroid (center of mass assuming uniform density).
    pub fn compute_centroid(&self) -> Point3<f64> {
        let mut centroid = Vector3::zeros();
        let mut total_area = 0.0;

        for face in &self.faces {
            let v0 = &self.vertices[face[0]];
            let v1 = &self.vertices[face[1]];
            let v2 = &self.vertices[face[2]];

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let area = edge1.cross(&edge2).norm() / 2.0;
            
            let face_centroid = (v0.coords + v1.coords + v2.coords) / 3.0;
            centroid += face_centroid * area;
            total_area += area;
        }

        Point3::from(centroid / total_area)
    }
}
