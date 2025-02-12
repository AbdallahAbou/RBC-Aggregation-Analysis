//! Benchmarks for mesh processing operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nalgebra::Point3;

fn create_test_mesh(n_vertices: usize) -> (Vec<Point3<f64>>, Vec<[usize; 3]>) {
    let vertices: Vec<Point3<f64>> = (0..n_vertices)
        .map(|i| {
            let t = i as f64 / n_vertices as f64 * std::f64::consts::TAU;
            Point3::new(t.cos(), t.sin(), i as f64 * 0.1)
        })
        .collect();
    
    let faces: Vec<[usize; 3]> = (0..n_vertices - 2)
        .map(|i| [0, i + 1, i + 2])
        .collect();
    
    (vertices, faces)
}

fn bench_volume_computation(c: &mut Criterion) {
    let (vertices, faces) = create_test_mesh(1000);
    
    c.bench_function("volume_1000_vertices", |b| {
        b.iter(|| {
            let mut volume = 0.0f64;
            for face in &faces {
                let v0 = &vertices[face[0]];
                let v1 = &vertices[face[1]];
                let v2 = &vertices[face[2]];
                volume += v0.coords.dot(&v1.coords.cross(&v2.coords));
            }
            black_box((volume / 6.0).abs())
        })
    });
}

criterion_group!(benches, bench_volume_computation);
criterion_main!(benches);
