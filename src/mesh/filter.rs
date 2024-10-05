//! Mesh filtering and quality operations.

use crate::mesh::Mesh;

/// Mesh quality filters.
pub struct MeshFilter;

impl MeshFilter {
    /// Remove small disconnected components.
    pub fn remove_small_components(mesh: &mut Mesh, min_vertices: usize) {
        if mesh.vertices.len() < min_vertices {
            mesh.vertices.clear();
            mesh.faces.clear();
        }
    }
    
    /// Remove degenerate triangles (zero area).
    pub fn remove_degenerate_faces(mesh: &mut Mesh, area_threshold: f64) {
        mesh.faces.retain(|face| {
            let v0 = &mesh.vertices[face[0]];
            let v1 = &mesh.vertices[face[1]];
            let v2 = &mesh.vertices[face[2]];
            
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let area = edge1.cross(&edge2).norm() / 2.0;
            
            area > area_threshold
        });
    }
    
    /// Smooth mesh using Laplacian smoothing.
    pub fn laplacian_smooth(mesh: &mut Mesh, iterations: usize, lambda: f64) {
        use std::collections::HashMap;
        
        // Build adjacency
        let mut neighbors: HashMap<usize, Vec<usize>> = HashMap::new();
        for face in &mesh.faces {
            for i in 0..3 {
                let v = face[i];
                let n1 = face[(i + 1) % 3];
                let n2 = face[(i + 2) % 3];
                neighbors.entry(v).or_default().push(n1);
                neighbors.entry(v).or_default().push(n2);
            }
        }
        
        for _ in 0..iterations {
            let old_vertices = mesh.vertices.clone();
            
            for (i, vertex) in mesh.vertices.iter_mut().enumerate() {
                if let Some(nbrs) = neighbors.get(&i) {
                    let centroid: nalgebra::Vector3<f64> = nbrs.iter()
                        .map(|&j| old_vertices[j].coords)
                        .sum::<nalgebra::Vector3<f64>>() / nbrs.len() as f64;
                    
                    vertex.coords = vertex.coords * (1.0 - lambda) + centroid * lambda;
                }
            }
        }
    }
}
