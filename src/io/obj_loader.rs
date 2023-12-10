//! OBJ file loader for triangular meshes.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use nalgebra::Point3;
use crate::mesh::Mesh;

/// Loader for Wavefront OBJ mesh files.
pub struct ObjLoader;

impl ObjLoader {
    /// Load a mesh from an OBJ file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Mesh, std::io::Error> {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        
        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            if parts.is_empty() {
                continue;
            }
            
            match parts[0] {
                "v" => {
                    let x: f64 = parts[1].parse().unwrap_or(0.0);
                    let y: f64 = parts[2].parse().unwrap_or(0.0);
                    let z: f64 = parts[3].parse().unwrap_or(0.0);
                    vertices.push(Point3::new(x, y, z));
                }
                "f" => {
                    // Parse face indices (OBJ is 1-indexed)
                    let indices: Vec<usize> = parts[1..].iter()
                        .filter_map(|p| p.split('/').next())
                        .filter_map(|s| s.parse::<usize>().ok())
                        .map(|i| i - 1)
                        .collect();
                    
                    if indices.len() >= 3 {
                        faces.push([indices[0], indices[1], indices[2]]);
                    }
                }
                _ => {}
            }
        }
        
        let id = path.as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        Ok(Mesh::new(vertices, faces, id))
    }
}
