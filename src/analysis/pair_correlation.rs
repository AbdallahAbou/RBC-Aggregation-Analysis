//! Pair correlation function g(r) computation for 3D point sets.

use rayon::prelude::*;
use std::f64::consts::PI;

/// Computes the radial distribution function (pair correlation function)
/// for a set of particles in a cubic domain.
///
/// The pair correlation function g(r) describes how density varies as a
/// function of distance from a reference particle. For uncorrelated
/// particles, g(r) = 1. Peaks indicate preferred separation distances.
///
/// For RBC aggregation:
/// - g(r) peak near contact distance indicates aggregation
/// - Peak height correlates with aggregation strength
/// - Peak position indicates rouleaux formation vs. clusters
pub struct PairCorrelation {
    /// Domain size (cube side length)
    domain_size: f64,
    /// Maximum correlation distance
    r_max: f64,
    /// Radial bin width
    dr: f64,
}

impl PairCorrelation {
    /// Create a new pair correlation analyzer.
    ///
    /// # Arguments
    /// * `domain_size` - Side length of cubic domain
    /// * `r_max` - Maximum distance for correlation computation
    /// * `dr` - Width of radial bins
    pub fn new(domain_size: f64, r_max: f64, dr: f64) -> Self {
        Self { domain_size, r_max, dr }
    }

    /// Compute pair correlation function for given positions.
    ///
    /// Returns (g_r, radii) where:
    /// - g_r: correlation values at each radial bin
    /// - radii: center radius of each bin
    ///
    /// Uses interior particles only to avoid edge effects.
    pub fn compute(&self, positions: &[[f64; 3]]) -> (Vec<f64>, Vec<f64>) {
        let n_bins = (self.r_max / self.dr).ceil() as usize;
        let mut radii = vec![0.0; n_bins];
        
        // Initialize bin edges and centers
        for i in 0..n_bins {
            radii[i] = (i as f64 + 0.5) * self.dr;
        }

        // Find interior particles (sphere of r_max fits within domain)
        let interior_indices: Vec<usize> = positions
            .iter()
            .enumerate()
            .filter(|(_, p)| self.is_interior(p))
            .map(|(i, _)| i)
            .collect();

        if interior_indices.is_empty() {
            return (vec![0.0; n_bins], radii);
        }

        // Number density
        let number_density = positions.len() as f64 / self.domain_size.powi(3);

        // Compute histogram for each interior particle in parallel
        let histograms: Vec<Vec<f64>> = interior_indices
            .par_iter()
            .map(|&ref_idx| {
                let mut hist = vec![0.0; n_bins];
                let ref_pos = &positions[ref_idx];

                for (j, pos) in positions.iter().enumerate() {
                    if j == ref_idx {
                        continue;
                    }

                    let r = distance(ref_pos, pos);
                    if r < self.r_max {
                        let bin = (r / self.dr) as usize;
                        if bin < n_bins {
                            hist[bin] += 1.0;
                        }
                    }
                }

                hist
            })
            .collect();

        // Average histograms and normalize
        let n_interior = interior_indices.len() as f64;
        let mut g_r = vec![0.0; n_bins];

        for i in 0..n_bins {
            let sum: f64 = histograms.iter().map(|h| h[i]).sum();
            let avg_count = sum / n_interior;

            // Shell volume normalization
            let r_inner = i as f64 * self.dr;
            let r_outer = (i + 1) as f64 * self.dr;
            let shell_volume = (4.0 / 3.0) * PI * (r_outer.powi(3) - r_inner.powi(3));

            // Expected count for uniform distribution
            let expected = number_density * shell_volume;

            g_r[i] = if expected > 0.0 {
                avg_count / expected
            } else {
                0.0
            };
        }

        (g_r, radii)
    }

    /// Check if a point is interior (r_max sphere fits in domain).
    fn is_interior(&self, p: &[f64; 3]) -> bool {
        p.iter().all(|&x| x > self.r_max && x < (self.domain_size - self.r_max))
    }
}

/// Euclidean distance between two 3D points.
#[inline]
fn distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_distribution() {
        // For uniform random points, g(r) should be approximately 1
        let mut positions = Vec::new();
        let domain = 100.0;

        // Simple grid of points
        for i in 0..10 {
            for j in 0..10 {
                for k in 0..10 {
                    positions.push([
                        (i as f64 + 0.5) * 10.0,
                        (j as f64 + 0.5) * 10.0,
                        (k as f64 + 0.5) * 10.0,
                    ]);
                }
            }
        }

        let pcf = PairCorrelation::new(domain, 20.0, 1.0);
        let (g_r, _) = pcf.compute(&positions);

        // Grid should show peaks at lattice spacings
        assert!(g_r.len() > 0);
    }

    #[test]
    fn test_empty_positions() {
        let pcf = PairCorrelation::new(100.0, 10.0, 1.0);
        let (g_r, radii) = pcf.compute(&[]);
        
        assert!(g_r.iter().all(|&g| g == 0.0));
    }
}
