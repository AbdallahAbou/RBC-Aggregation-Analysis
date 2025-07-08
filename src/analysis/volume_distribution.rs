//! Volume distribution analysis for cell populations.

/// Statistics for volume distribution.
pub struct VolumeDistribution {
    pub volumes: Vec<f64>,
    pub mean: f64,
    pub std_dev: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
}

impl VolumeDistribution {
    pub fn from_volumes(mut volumes: Vec<f64>) -> Self {
        volumes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let n = volumes.len() as f64;
        let mean = volumes.iter().sum::<f64>() / n;
        let variance = volumes.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / n;
        let std_dev = variance.sqrt();
        
        let median = if volumes.len() % 2 == 0 {
            (volumes[volumes.len() / 2 - 1] + volumes[volumes.len() / 2]) / 2.0
        } else {
            volumes[volumes.len() / 2]
        };
        
        Self {
            min: *volumes.first().unwrap_or(&0.0),
            max: *volumes.last().unwrap_or(&0.0),
            volumes,
            mean,
            std_dev,
            median,
        }
    }
    
    /// Compute histogram of volumes.
    pub fn histogram(&self, n_bins: usize) -> (Vec<f64>, Vec<usize>) {
        let range = self.max - self.min;
        let bin_width = range / n_bins as f64;
        
        let edges: Vec<f64> = (0..=n_bins)
            .map(|i| self.min + i as f64 * bin_width)
            .collect();
        
        let mut counts = vec![0usize; n_bins];
        for &v in &self.volumes {
            let bin = ((v - self.min) / bin_width) as usize;
            let bin = bin.min(n_bins - 1);
            counts[bin] += 1;
        }
        
        (edges, counts)
    }
}

impl VolumeDistribution {
    /// Export statistics to JSON.
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"mean":{:.4},"std_dev":{:.4},"median":{:.4},"min":{:.4},"max":{:.4},"count":{}}}"#,
            self.mean, self.std_dev, self.median, self.min, self.max, self.volumes.len()
        )
    }
}
