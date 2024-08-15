//! Pair correlation function g(r) for spatial analysis.

use nalgebra::Point3;

/// Computes radial distribution function g(r).
pub struct PairCorrelation {
    /// Maximum distance to compute
    pub r_max: f64,
    /// Number of bins
    pub n_bins: usize,
    /// Bin edges
    pub bin_edges: Vec<f64>,
    /// g(r) values
    pub values: Vec<f64>,
}

impl PairCorrelation {
    pub fn new(r_max: f64, n_bins: usize) -> Self {
        let dr = r_max / n_bins as f64;
        let bin_edges: Vec<f64> = (0..=n_bins).map(|i| i as f64 * dr).collect();
        
        Self {
            r_max,
            n_bins,
            bin_edges,
            values: vec![0.0; n_bins],
        }
    }
    
    /// Compute g(r) from a set of cell centroids.
    pub fn compute(&mut self, centroids: &[Point3<f64>], box_volume: f64) {
        let n = centroids.len();
        let density = n as f64 / box_volume;
        let dr = self.r_max / self.n_bins as f64;
        
        let mut hist = vec![0usize; self.n_bins];
        
        for i in 0..n {
            for j in (i + 1)..n {
                let dist = (centroids[i] - centroids[j]).norm();
                if dist < self.r_max {
                    let bin = (dist / dr) as usize;
                    if bin < self.n_bins {
                        hist[bin] += 2; // Count both i-j and j-i
                    }
                }
            }
        }
        
        // Normalize by shell volume and ideal gas
        for (k, count) in hist.iter().enumerate() {
            let r_inner = k as f64 * dr;
            let r_outer = (k + 1) as f64 * dr;
            let shell_vol = 4.0 / 3.0 * std::f64::consts::PI 
                * (r_outer.powi(3) - r_inner.powi(3));
            let ideal = density * shell_vol * n as f64;
            self.values[k] = *count as f64 / ideal;
        }
    }
}
