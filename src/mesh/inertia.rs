//! Inertia tensor properties and shape analysis.

use nalgebra::Matrix3;

/// Principal inertia properties of a 3D body.
#[derive(Debug, Clone)]
pub struct InertiaProperties {
    /// Largest principal moment
    pub lambda1: f64,
    /// Middle principal moment
    pub lambda2: f64,
    /// Smallest principal moment
    pub lambda3: f64,
    /// Principal axes as column vectors
    pub principal_axes: Matrix3<f64>,
}

impl Default for InertiaProperties {
    fn default() -> Self {
        Self {
            lambda1: 0.0,
            lambda2: 0.0,
            lambda3: 0.0,
            principal_axes: Matrix3::identity(),
        }
    }
}

impl InertiaProperties {
    /// Compute shape descriptors from eigenvalues.
    ///
    /// Returns (asphericity, acylindricity):
    /// - Asphericity: deviation from spherical symmetry
    /// - Acylindricity: deviation from cylindrical symmetry
    ///
    /// For RBCs:
    /// - Healthy discocyte: high asphericity, low acylindricity
    /// - Spherocyte: low asphericity, low acylindricity
    /// - Elliptocyte: high asphericity, high acylindricity
    pub fn shape_descriptors(&self) -> (f64, f64) {
        let sum = self.lambda1 + self.lambda2 + self.lambda3;
        if sum == 0.0 {
            return (0.0, 0.0);
        }

        // Asphericity: b = λ1 - 0.5(λ2 + λ3)
        let asphericity = self.lambda1 - 0.5 * (self.lambda2 + self.lambda3);

        // Acylindricity: c = λ2 - λ3
        let acylindricity = self.lambda2 - self.lambda3;

        // Normalize by trace
        (asphericity / sum, acylindricity / sum)
    }

    /// Ratio of largest to middle eigenvalue.
    /// Used for RBC shape classification.
    pub fn lambda_ratio_12(&self) -> f64 {
        if self.lambda2 == 0.0 {
            return f64::INFINITY;
        }
        self.lambda1 / self.lambda2
    }

    /// Ratio of middle to smallest eigenvalue.
    pub fn lambda_ratio_23(&self) -> f64 {
        if self.lambda3 == 0.0 {
            return f64::INFINITY;
        }
        self.lambda2 / self.lambda3
    }

    /// Check if eigenvalue ratios indicate valid RBC geometry.
    ///
    /// Valid RBC should have:
    /// - λ2 + λ3 ≈ λ1 (within 25%)
    /// - λ2 ≈ λ3 (within factor of 2)
    pub fn is_valid_rbc_geometry(&self) -> bool {
        let sum_23 = self.lambda2 + self.lambda3;
        let tolerance = self.lambda1 * 0.25;

        let sum_check = (sum_23 - self.lambda1).abs() < tolerance;
        let ratio_check = self.lambda2 >= self.lambda3 / 2.0 
            && self.lambda2 <= self.lambda3 * 2.0;

        sum_check && ratio_check
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_descriptors_sphere() {
        // Sphere has equal eigenvalues
        let props = InertiaProperties {
            lambda1: 1.0,
            lambda2: 1.0,
            lambda3: 1.0,
            principal_axes: Matrix3::identity(),
        };

        let (asp, acyl) = props.shape_descriptors();
        assert!(asp.abs() < 1e-10);
        assert!(acyl.abs() < 1e-10);
    }

    #[test]
    fn test_shape_descriptors_oblate() {
        // Oblate spheroid (disc-like): λ1 > λ2 ≈ λ3
        let props = InertiaProperties {
            lambda1: 2.0,
            lambda2: 1.0,
            lambda3: 1.0,
            principal_axes: Matrix3::identity(),
        };

        let (asp, acyl) = props.shape_descriptors();
        assert!(asp > 0.0);
        assert!(acyl.abs() < 1e-10);
    }
}
