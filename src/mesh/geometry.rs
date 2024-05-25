//! Core mesh geometry representation and operations.

use nalgebra::{Matrix3, Point3, Vector3};
use std::path::Path;

use super::{CellRecord, InertiaProperties};

/// Triangular mesh representation of a 3D cell.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Vertex positions
    pub vertices: Vec<Point3<f64>>,
    /// Triangle indices (i, j, k) referencing vertices
    pub faces: Vec<[usize; 3]>,
    /// Source file identifier
    pub id: String,
    /// Cached geometric properties
    properties: Option<MeshProperties>,
}

#[derive(Debug, Clone)]
struct MeshProperties {
    volume: f64,
    centroid: Point3<f64>,
    inertia: InertiaProperties,
    is_watertight: bool,
}

impl Mesh {
    /// Create a new mesh from vertices and faces.
    pub fn new(vertices: Vec<Point3<f64>>, faces: Vec<[usize; 3]>, id: String) -> Self {
        let mut mesh = Self {
            vertices,
            faces,
            id,
            properties: None,
        };
        mesh.compute_properties();
        mesh
    }

    /// Compute and cache all geometric properties.
    fn compute_properties(&mut self) {
        let volume = self.compute_volume();
        let centroid = self.compute_centroid();
        let inertia = self.compute_inertia_tensor();
        let is_watertight = self.check_watertight();

        self.properties = Some(MeshProperties {
            volume,
            centroid,
            inertia,
            is_watertight,
        });
    }

    /// Signed volume of mesh using divergence theorem.
    /// For closed meshes, returns the enclosed volume.
    fn compute_volume(&self) -> f64 {
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

    /// Compute centroid (center of mass assuming uniform density).
    fn compute_centroid(&self) -> Point3<f64> {
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

    /// Compute inertia tensor and extract principal components.
    fn compute_inertia_tensor(&self) -> InertiaProperties {
        let centroid = self.compute_centroid();
        let mut inertia = Matrix3::zeros();

        // Approximate inertia using vertex distribution
        for vertex in &self.vertices {
            let r = vertex - centroid;
            let r2 = r.norm_squared();

            // Diagonal elements
            inertia[(0, 0)] += r2 - r.x * r.x;
            inertia[(1, 1)] += r2 - r.y * r.y;
            inertia[(2, 2)] += r2 - r.z * r.z;

            // Off-diagonal elements
            inertia[(0, 1)] -= r.x * r.y;
            inertia[(0, 2)] -= r.x * r.z;
            inertia[(1, 2)] -= r.y * r.z;
        }

        // Symmetric matrix
        inertia[(1, 0)] = inertia[(0, 1)];
        inertia[(2, 0)] = inertia[(0, 2)];
        inertia[(2, 1)] = inertia[(1, 2)];

        // Eigendecomposition for principal axes
        let eigen = inertia.symmetric_eigen();
        let mut eigenvalues: Vec<f64> = eigen.eigenvalues.iter().copied().collect();
        eigenvalues.sort_by(|a, b| b.partial_cmp(a).unwrap());

        InertiaProperties {
            lambda1: eigenvalues[0],
            lambda2: eigenvalues[1],
            lambda3: eigenvalues[2],
            principal_axes: eigen.eigenvectors,
        }
    }

    /// Check if mesh is watertight (closed manifold).
    fn check_watertight(&self) -> bool {
        use std::collections::HashMap;

        // Count edge occurrences - each edge should appear exactly twice
        let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

        for face in &self.faces {
            for i in 0..3 {
                let v0 = face[i];
                let v1 = face[(i + 1) % 3];
                let edge = if v0 < v1 { (v0, v1) } else { (v1, v0) };
                *edge_count.entry(edge).or_insert(0) += 1;
            }
        }

        edge_count.values().all(|&count| count == 2)
    }

    /// Get mesh volume in physical units.
    pub fn volume(&self) -> f64 {
        self.properties.as_ref().map_or(0.0, |p| p.volume)
    }

    /// Get mesh centroid.
    pub fn centroid(&self) -> Point3<f64> {
        self.properties
            .as_ref()
            .map_or(Point3::origin(), |p| p.centroid)
    }

    /// Get principal inertia components.
    pub fn inertia(&self) -> &InertiaProperties {
        static DEFAULT: std::sync::OnceLock<InertiaProperties> = std::sync::OnceLock::new();
        self.properties
            .as_ref()
            .map_or_else(
                || DEFAULT.get_or_init(InertiaProperties::default),
                |p| &p.inertia
            )
    }

    /// Check if mesh is watertight.
    pub fn is_watertight(&self) -> bool {
        self.properties.as_ref().map_or(false, |p| p.is_watertight)
    }

    /// Check if centroid has all positive coordinates.
    pub fn has_positive_centroid(&self) -> bool {
        let c = self.centroid();
        c.x > 0.0 && c.y > 0.0 && c.z > 0.0
    }

    /// Convert mesh properties to exportable record.
    /// Applies voxel-to-physical unit conversion.
    pub fn to_cell_record(&self) -> CellRecord {
        // Voxel size conversion factors (um)
        const VOXEL_XY: f64 = 0.241;
        const VOXEL_Z: f64 = 0.334;

        let centroid = self.centroid();
        let inertia = self.inertia();
        let volume = self.volume() * VOXEL_XY * VOXEL_XY * VOXEL_Z;

        // Shape descriptors from gyration tensor eigenvalues
        let (asphericity, acylindricity) = inertia.shape_descriptors();

        CellRecord {
            id: self.id.clone(),
            x: centroid.x * VOXEL_XY,
            y: centroid.y * VOXEL_XY,
            z: centroid.z * VOXEL_Z,
            lambda1: inertia.lambda1 / 1000.0,
            lambda2: inertia.lambda2 / 1000.0,
            lambda3: inertia.lambda3 / 1000.0,
            asphericity,
            acylindricity,
            volume,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_mesh() -> Mesh {
        // Unit cube centered at (0.5, 0.5, 0.5)
        let vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(0.0, 1.0, 1.0),
        ];

        let faces = vec![
            [0, 2, 1], [0, 3, 2], // bottom
            [4, 5, 6], [4, 6, 7], // top
            [0, 1, 5], [0, 5, 4], // front
            [2, 3, 7], [2, 7, 6], // back
            [0, 4, 7], [0, 7, 3], // left
            [1, 2, 6], [1, 6, 5], // right
        ];

        Mesh::new(vertices, faces, "test_cube".to_string())
    }

    #[test]
    fn test_volume_calculation() {
        let mesh = cube_mesh();
        assert!((mesh.volume() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_watertight_check() {
        let mesh = cube_mesh();
        assert!(mesh.is_watertight());
    }

    #[test]
    fn test_centroid() {
        let mesh = cube_mesh();
        let c = mesh.centroid();
        assert!((c.x - 0.5).abs() < 1e-10);
        assert!((c.y - 0.5).abs() < 1e-10);
        assert!((c.z - 0.5).abs() < 1e-10);
    }
}
