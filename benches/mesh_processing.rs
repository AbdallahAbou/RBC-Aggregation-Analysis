use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nalgebra::Point3;

fn create_test_mesh(n_vertices: usize) -> (Vec<Point3<f64>>, Vec<[usize; 3]>) {
    let mut vertices = Vec::with_capacity(n_vertices);
    let mut faces = Vec::new();

    // Create sphere-like point cloud
    for i in 0..n_vertices {
        let phi = (i as f64 / n_vertices as f64) * std::f64::consts::PI * 2.0;
        let theta = (i as f64 / n_vertices as f64) * std::f64::consts::PI;
        
        let x = theta.sin() * phi.cos();
        let y = theta.sin() * phi.sin();
        let z = theta.cos();
        
        vertices.push(Point3::new(x, y, z));
    }

    // Simple triangulation
    for i in 0..(n_vertices - 2) {
        faces.push([0, i + 1, i + 2]);
    }

    (vertices, faces)
}

fn bench_volume_calculation(c: &mut Criterion) {
    let (vertices, faces) = create_test_mesh(1000);

    c.bench_function("volume_1000_vertices", |b| {
        b.iter(|| {
            let mut volume = 0.0;
            for face in &faces {
                let v0 = &vertices[face[0]];
                let v1 = &vertices[face[1]];
                let v2 = &vertices[face[2]];
                volume += v0.coords.dot(&v1.coords.cross(&v2.coords));
            }
            black_box(volume / 6.0)
        })
    });
}

fn bench_centroid_calculation(c: &mut Criterion) {
    let (vertices, faces) = create_test_mesh(1000);

    c.bench_function("centroid_1000_vertices", |b| {
        b.iter(|| {
            let mut centroid = nalgebra::Vector3::zeros();
            let mut total_area = 0.0;

            for face in &faces {
                let v0 = &vertices[face[0]];
                let v1 = &vertices[face[1]];
                let v2 = &vertices[face[2]];

                let edge1 = v1 - v0;
                let edge2 = v2 - v0;
                let area = edge1.cross(&edge2).norm() / 2.0;

                let face_centroid = (v0.coords + v1.coords + v2.coords) / 3.0;
                centroid += face_centroid * area;
                total_area += area;
            }

            black_box(centroid / total_area)
        })
    });
}

criterion_group!(benches, bench_volume_calculation, bench_centroid_calculation);
criterion_main!(benches);
