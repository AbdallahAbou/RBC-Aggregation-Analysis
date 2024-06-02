//! Filtering criteria for valid RBC meshes.

use super::Mesh;

/// Builder for mesh validation filters.
///
/// Filters out meshes that don't meet geometric criteria
/// for valid red blood cell segmentation.
#[derive(Debug, Clone)]
pub struct MeshFilter {
    min_volume: Option<f64>,
    max_volume: Option<f64>,
    require_watertight: bool,
    require_positive_centroid: bool,
    require_valid_inertia: bool,
}

impl Default for MeshFilter {
    fn default() -> Self {
        Self {
            min_volume: None,
            max_volume: None,
            require_watertight: false,
            require_positive_centroid: false,
            require_valid_inertia: false,
        }
    }
}

impl MeshFilter {
    /// Create a new filter with no constraints.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set volume bounds (in physical units, um^3).
    ///
    /// Typical healthy RBC volume: 80-100 um^3
    /// Filter range accounts for segmentation variation.
    pub fn with_volume_bounds(mut self, min: f64, max: f64) -> Self {
        self.min_volume = Some(min);
        self.max_volume = Some(max);
        self
    }

    /// Require mesh to be watertight (closed manifold).
    ///
    /// Non-watertight meshes indicate incomplete segmentation
    /// or cells touching the image boundary.
    pub fn require_watertight(mut self) -> Self {
        self.require_watertight = true;
        self
    }

    /// Require centroid to have positive coordinates.
    ///
    /// Cells with negative centroid coordinates are on the
    /// edge of the imaging volume and may be incomplete.
    pub fn require_positive_centroid(mut self) -> Self {
        self.require_positive_centroid = true;
        self
    }

    /// Require valid RBC inertia geometry.
    ///
    /// Filters out debris and incorrectly segmented objects
    /// based on expected eigenvalue relationships.
    pub fn require_valid_inertia(mut self) -> Self {
        self.require_valid_inertia = true;
        self
    }

    /// Check if a mesh passes all filter criteria.
    pub fn accepts(&self, mesh: &Mesh) -> bool {
        // Volume check
        if let Some(min) = self.min_volume {
            if mesh.volume() < min {
                return false;
            }
        }
        if let Some(max) = self.max_volume {
            if mesh.volume() > max {
                return false;
            }
        }

        // Watertight check
        if self.require_watertight && !mesh.is_watertight() {
            return false;
        }

        // Positive centroid check
        if self.require_positive_centroid && !mesh.has_positive_centroid() {
            return false;
        }

        // Inertia geometry check
        if self.require_valid_inertia && !mesh.inertia().is_valid_rbc_geometry() {
            return false;
        }

        true
    }
}

/// Statistics about filtered mesh processing.
#[derive(Debug, Default)]
pub struct FilterStats {
    pub total: usize,
    pub passed: usize,
    pub rejected_volume: usize,
    pub rejected_watertight: usize,
    pub rejected_centroid: usize,
    pub rejected_inertia: usize,
}

impl FilterStats {
    /// Acceptance ratio.
    pub fn acceptance_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.passed as f64 / self.total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;

    fn dummy_mesh(volume: f64, watertight: bool) -> Mesh {
        // Create minimal mesh for testing
        let scale = volume.cbrt();
        let vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(scale, 0.0, 0.0),
            Point3::new(0.0, scale, 0.0),
            Point3::new(0.0, 0.0, scale),
        ];

        let faces = if watertight {
            vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]]
        } else {
            vec![[0, 1, 2], [0, 2, 3]]
        };

        Mesh::new(vertices, faces, "test".to_string())
    }

    #[test]
    fn test_volume_filter() {
        let filter = MeshFilter::new().with_volume_bounds(50.0, 150.0);
        
        // These would need actual volume-correct meshes in real tests
        // Simplified for demonstration
    }
}
