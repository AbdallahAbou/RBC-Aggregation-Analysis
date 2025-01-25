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

impl InertiaProperties {
    /// Compute asphericity and prolateness from principal moments.
    pub fn compute_shape_descriptors(moments: &Vector3<f64>) -> (f64, f64) {
        let mut sorted: Vec<f64> = moments.iter().cloned().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let (l1, l2, l3) = (sorted[0], sorted[1], sorted[2]);
        let mean = (l1 + l2 + l3) / 3.0;
        
        // Asphericity: 0 for sphere, 1 for rod
        let asphericity = ((l1 - mean).powi(2) + (l2 - mean).powi(2) + (l3 - mean).powi(2))
            / (2.0 * mean.powi(2));
        
        // Prolateness: -1 for oblate, +1 for prolate
        let prolateness = (2.0 * l2 - l1 - l3) / (l3 - l1);
        
        (asphericity, prolateness)
    }
}
