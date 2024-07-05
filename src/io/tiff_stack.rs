//! TIFF stack reader for 3D microscopy images.

use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use thiserror::Error;
use tiff::decoder::{Decoder, DecodingResult};
use tiff::ColorType;

#[derive(Error, Debug)]
pub enum TiffError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TIFF decoding error: {0}")]
    Decode(#[from] tiff::TiffError),
    
    #[error("Unsupported color type: {0:?}")]
    UnsupportedColorType(ColorType),
    
    #[error("Invalid slice range: {start}..{end} (max: {max})")]
    InvalidRange { start: usize, end: usize, max: usize },
}

/// Reader for multi-page TIFF stacks (3D confocal microscopy data).
///
/// Confocal microscopy produces Z-stacks where each slice is a
/// 2D image at a different focal depth. This reader extracts
/// intensity information for quality assessment and segmentation.
pub struct TiffStack {
    path: std::path::PathBuf,
    num_slices: usize,
    width: u32,
    height: u32,
}

impl TiffStack {
    /// Open a TIFF stack file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, TiffError> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut decoder = Decoder::new(reader)?;

        let (width, height) = decoder.dimensions()?;

        // Count slices by iterating through pages
        let mut num_slices = 1;
        while decoder.next_image().is_ok() {
            num_slices += 1;
        }

        Ok(Self {
            path: path.to_path_buf(),
            num_slices,
            width,
            height,
        })
    }

    /// Number of slices in the stack.
    pub fn num_slices(&self) -> usize {
        self.num_slices
    }

    /// Image dimensions (width, height).
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Compute mean intensity for each slice in range.
    ///
    /// Used for:
    /// - Identifying optimal focal planes
    /// - Quality control (detecting empty/saturated slices)
    /// - Background correction
    pub fn compute_slice_intensities(
        &self,
        start: usize,
        end: usize,
    ) -> Result<Vec<f64>, TiffError> {
        if end > self.num_slices || start >= end {
            return Err(TiffError::InvalidRange {
                start,
                end,
                max: self.num_slices,
            });
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut decoder = Decoder::new(reader)?;

        let mut intensities = Vec::with_capacity(end - start);

        // Skip to start slice
        for _ in 0..start {
            decoder.next_image()?;
        }

        // Process slices in range
        for _ in start..end {
            let intensity = self.compute_slice_mean(&mut decoder)?;
            intensities.push(intensity);
            
            if decoder.next_image().is_err() {
                break;
            }
        }

        Ok(intensities)
    }

    /// Compute mean intensity of current slice.
    fn compute_slice_mean<R: std::io::Read + std::io::Seek>(
        &self,
        decoder: &mut Decoder<R>,
    ) -> Result<f64, TiffError> {
        let color_type = decoder.colortype()?;
        let result = decoder.read_image()?;

        let mean = match result {
            DecodingResult::U8(data) => {
                data.iter().map(|&v| v as f64).sum::<f64>() / data.len() as f64
            }
            DecodingResult::U16(data) => {
                data.iter().map(|&v| v as f64).sum::<f64>() / data.len() as f64
            }
            DecodingResult::U32(data) => {
                data.iter().map(|&v| v as f64).sum::<f64>() / data.len() as f64
            }
            DecodingResult::F32(data) => {
                data.iter().map(|&v| v as f64).sum::<f64>() / data.len() as f64
            }
            DecodingResult::F64(data) => {
                data.iter().sum::<f64>() / data.len() as f64
            }
            _ => return Err(TiffError::UnsupportedColorType(color_type)),
        };

        Ok(mean)
    }

    /// Find slices with intensity above threshold (region of interest).
    ///
    /// Useful for automatically detecting the sample region
    /// in large Z-stacks with empty space above/below.
    pub fn find_roi_slices(&self, threshold_percentile: f64) -> Result<(usize, usize), TiffError> {
        let intensities = self.compute_slice_intensities(0, self.num_slices)?;
        
        let max_intensity = intensities.iter().cloned().fold(0.0, f64::max);
        let threshold = max_intensity * threshold_percentile / 100.0;

        let start = intensities
            .iter()
            .position(|&i| i > threshold)
            .unwrap_or(0);

        let end = intensities
            .iter()
            .rposition(|&i| i > threshold)
            .map(|i| i + 1)
            .unwrap_or(self.num_slices);

        Ok((start, end))
    }
}
