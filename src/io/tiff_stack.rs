//! TIFF stack loader for 3D microscopy data.

use std::path::Path;
use anyhow::Result;

/// Loader for multi-page TIFF stacks from confocal microscopy.
pub struct TiffStack {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub data: Vec<u16>,
    pub voxel_size: [f64; 3],
}

impl TiffStack {
    /// Load a TIFF stack from file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        use tiff::decoder::Decoder;
        use std::fs::File;
        
        let file = File::open(path)?;
        let mut decoder = Decoder::new(file)?;
        
        let (width, height) = decoder.dimensions()?;
        let mut slices = Vec::new();
        
        loop {
            let image = decoder.read_image()?;
            if let tiff::decoder::DecodingResult::U16(data) = image {
                slices.push(data);
            }
            
            if decoder.more_images() {
                decoder.next_image()?;
            } else {
                break;
            }
        }
        
        let depth = slices.len();
        let data: Vec<u16> = slices.into_iter().flatten().collect();
        
        Ok(Self {
            width: width as usize,
            height: height as usize,
            depth,
            data,
            voxel_size: [1.0, 1.0, 1.0],
        })
    }
    
    /// Get voxel value at (x, y, z).
    pub fn get(&self, x: usize, y: usize, z: usize) -> u16 {
        let idx = z * self.width * self.height + y * self.width + x;
        self.data.get(idx).copied().unwrap_or(0)
    }
}
