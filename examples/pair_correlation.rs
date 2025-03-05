//! Example: Compute pair correlation from random points.

use nalgebra::Point3;
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let box_size = 100.0;
    let n_points = 500;
    
    // Generate random points
    let centroids: Vec<Point3<f64>> = (0..n_points)
        .map(|_| Point3::new(
            rng.gen::<f64>() * box_size,
            rng.gen::<f64>() * box_size,
            rng.gen::<f64>() * box_size,
        ))
        .collect();
    
    println!("Generated {} random points in box {}^3", n_points, box_size);
    println!("Computing pair correlation g(r)...");
    
    // Simple g(r) computation
    let r_max = 20.0;
    let n_bins = 50;
    let dr = r_max / n_bins as f64;
    let density = n_points as f64 / box_size.powi(3);
    
    let mut hist = vec![0usize; n_bins];
    
    for i in 0..n_points {
        for j in (i + 1)..n_points {
            let dist = (centroids[i] - centroids[j]).norm();
            if dist < r_max {
                let bin = (dist / dr) as usize;
                if bin < n_bins {
                    hist[bin] += 2;
                }
            }
        }
    }
    
    println!("\nr\tg(r)");
    for k in 0..n_bins {
        let r = (k as f64 + 0.5) * dr;
        let r_inner = k as f64 * dr;
        let r_outer = (k + 1) as f64 * dr;
        let shell_vol = 4.0 / 3.0 * std::f64::consts::PI 
            * (r_outer.powi(3) - r_inner.powi(3));
        let ideal = density * shell_vol * n_points as f64;
        let gr = hist[k] as f64 / ideal;
        println!("{:.2}\t{:.4}", r, gr);
    }
}
