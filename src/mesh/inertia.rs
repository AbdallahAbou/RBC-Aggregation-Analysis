//! Inertia tensor computation for mesh geometries.

use nalgebra::{Matrix3, Vector3};

/// Inertia properties of a 3D mesh.
#[derive(Debug, Clone)]
pub struct InertiaProperties {
    /// Inertia tensor about the centroid
    pub tensor: Matrix3<f64>,
    /// Principal moments of inertia (eigenvalues)
    pub principal_moments: Vector3<f64>,
    /// Principal axes (eigenvectors as columns)
    pub principal_axes: Matrix3<f64>,
    /// Asphericity parameter (0 = sphere, 1 = rod)
    pub asphericity: f64,
    /// Prolateness (-1 = oblate, 1 = prolate)
    pub prolateness: f64,
}
