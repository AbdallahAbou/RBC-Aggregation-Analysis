//! Volume distribution analysis and statistics.

/// Analyzes the volume distribution of a cell population.
///
/// RBC volume is a key diagnostic parameter:
/// - Mean Corpuscular Volume (MCV): 80-100 fL for healthy adults
/// - High MCV: macrocytic anemia, B12/folate deficiency
/// - Low MCV: microcytic anemia, iron deficiency
pub struct VolumeDistribution {
    volumes: Vec<f64>,
    sorted: bool,
}

impl VolumeDistribution {
    /// Create a new volume distribution from measurements.
    pub fn new(volumes: Vec<f64>) -> Self {
        Self {
            volumes,
            sorted: false,
        }
    }

    /// Add a volume measurement.
    pub fn push(&mut self, volume: f64) {
        self.volumes.push(volume);
        self.sorted = false;
    }

    /// Number of measurements.
    pub fn len(&self) -> usize {
        self.volumes.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.volumes.is_empty()
    }

    /// Compute mean volume.
    pub fn mean(&self) -> f64 {
        if self.volumes.is_empty() {
            return 0.0;
        }
        self.volumes.iter().sum::<f64>() / self.volumes.len() as f64
    }

    /// Compute standard deviation.
    pub fn std_dev(&self) -> f64 {
        if self.volumes.len() < 2 {
            return 0.0;
        }

        let mean = self.mean();
        let variance = self.volumes
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / (self.volumes.len() - 1) as f64;

        variance.sqrt()
    }

    /// Compute coefficient of variation (RDW proxy).
    ///
    /// Red cell Distribution Width (RDW) measures size heterogeneity.
    /// Normal: 11.5-14.5%
    pub fn coefficient_of_variation(&self) -> f64 {
        let mean = self.mean();
        if mean == 0.0 {
            return 0.0;
        }
        100.0 * self.std_dev() / mean
    }

    /// Ensure volumes are sorted for percentile calculations.
    fn ensure_sorted(&mut self) {
        if !self.sorted {
            self.volumes.sort_by(|a, b| a.partial_cmp(b).unwrap());
            self.sorted = true;
        }
    }

    /// Compute percentile value.
    pub fn percentile(&mut self, p: f64) -> f64 {
        if self.volumes.is_empty() || p < 0.0 || p > 100.0 {
            return 0.0;
        }

        self.ensure_sorted();

        let idx = (p / 100.0 * (self.volumes.len() - 1) as f64).round() as usize;
        self.volumes[idx.min(self.volumes.len() - 1)]
    }

    /// Compute histogram bins for the distribution.
    pub fn histogram(&self, n_bins: usize) -> (Vec<f64>, Vec<usize>) {
        if self.volumes.is_empty() || n_bins == 0 {
            return (vec![], vec![]);
        }

        let min = self.volumes.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.volumes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        
        if min == max {
            return (vec![min], vec![self.volumes.len()]);
        }

        let bin_width = (max - min) / n_bins as f64;
        let mut counts = vec![0usize; n_bins];
        let mut centers = vec![0.0; n_bins];

        for i in 0..n_bins {
            centers[i] = min + (i as f64 + 0.5) * bin_width;
        }

        for &v in &self.volumes {
            let bin = ((v - min) / bin_width) as usize;
            let bin = bin.min(n_bins - 1);
            counts[bin] += 1;
        }

        (centers, counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        let dist = VolumeDistribution::new(vec![80.0, 90.0, 100.0, 90.0, 90.0]);
        assert!((dist.mean() - 90.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev() {
        let dist = VolumeDistribution::new(vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        // Known std dev for this dataset
        assert!((dist.std_dev() - 2.138).abs() < 0.01);
    }

    #[test]
    fn test_percentile() {
        let mut dist = VolumeDistribution::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        assert!((dist.percentile(50.0) - 5.5).abs() < 1.0);
    }
}
