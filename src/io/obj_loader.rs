//! Wavefront OBJ file loader for mesh data.

use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use nalgebra::Point3;
use thiserror::Error;

use crate::mesh::Mesh;

#[derive(Error, Debug)]
pub enum ObjError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },
    
    #[error("Invalid face index: {0}")]
    InvalidIndex(usize),
}

/// Loader for Wavefront OBJ mesh files.
///
/// Supports basic OBJ format with vertices (v) and faces (f).
/// Handles both triangular and quad faces (quads are triangulated).
pub struct ObjLoader;

impl ObjLoader {
    /// Load a mesh from an OBJ file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Mesh, ObjError> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "v" => {
                    if parts.len() < 4 {
                        return Err(ObjError::Parse {
                            line: line_num + 1,
                            message: "Vertex requires 3 coordinates".to_string(),
                        });
                    }

                    let x: f64 = parts[1].parse().map_err(|_| ObjError::Parse {
                        line: line_num + 1,
                        message: "Invalid x coordinate".to_string(),
                    })?;
                    let y: f64 = parts[2].parse().map_err(|_| ObjError::Parse {
                        line: line_num + 1,
                        message: "Invalid y coordinate".to_string(),
                    })?;
                    let z: f64 = parts[3].parse().map_err(|_| ObjError::Parse {
                        line: line_num + 1,
                        message: "Invalid z coordinate".to_string(),
                    })?;

                    vertices.push(Point3::new(x, y, z));
                }

                "f" => {
                    let indices: Result<Vec<usize>, _> = parts[1..]
                        .iter()
                        .map(|s| {
                            // Handle v/vt/vn format - extract just vertex index
                            let idx_str = s.split('/').next().unwrap_or(s);
                            idx_str.parse::<usize>().map(|i| i - 1) // OBJ is 1-indexed
                        })
                        .collect();

                    let indices = indices.map_err(|_| ObjError::Parse {
                        line: line_num + 1,
                        message: "Invalid face index".to_string(),
                    })?;

                    // Validate indices
                    for &idx in &indices {
                        if idx >= vertices.len() {
                            return Err(ObjError::InvalidIndex(idx));
                        }
                    }

                    // Triangulate if necessary
                    if indices.len() == 3 {
                        faces.push([indices[0], indices[1], indices[2]]);
                    } else if indices.len() == 4 {
                        // Quad -> two triangles
                        faces.push([indices[0], indices[1], indices[2]]);
                        faces.push([indices[0], indices[2], indices[3]]);
                    } else if indices.len() > 4 {
                        // Fan triangulation for polygons
                        for i in 1..indices.len() - 1 {
                            faces.push([indices[0], indices[i], indices[i + 1]]);
                        }
                    }
                }

                _ => {} // Ignore other elements (vt, vn, etc.)
            }
        }

        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Mesh::new(vertices, faces, id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_simple_cube() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Simple cube").unwrap();
        writeln!(file, "v 0 0 0").unwrap();
        writeln!(file, "v 1 0 0").unwrap();
        writeln!(file, "v 1 1 0").unwrap();
        writeln!(file, "v 0 1 0").unwrap();
        writeln!(file, "f 1 2 3").unwrap();
        writeln!(file, "f 1 3 4").unwrap();

        let mesh = ObjLoader::load(file.path()).unwrap();
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.faces.len(), 2);
    }
}
