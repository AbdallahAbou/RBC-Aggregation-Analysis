//! Example: Computing pair correlation function for RBC positions.

use rbc_aggregation::analysis::PairCorrelation;

fn main() {
    // Example cell positions from confocal microscopy
    // Coordinates in micrometers
    let positions = vec![
        [10.5, 20.3, 5.2],
        [15.2, 22.1, 5.8],
        [12.8, 18.9, 4.9],
        [45.0, 50.2, 10.1],
        [47.3, 51.8, 10.5],
        [46.1, 48.9, 9.8],
        [80.2, 30.5, 15.2],
        [82.1, 32.4, 15.8],
        [78.9, 31.2, 14.9],
    ];

    // Domain size: 120 um cube
    // Correlation up to 20 um
    // Bin width: 0.5 um
    let pcf = PairCorrelation::new(120.0, 20.0, 0.5);
    let (g_r, radii) = pcf.compute(&positions);

    println!("Pair Correlation Function g(r)");
    println!("================================");
    println!();
    println!("Interpretation:");
    println!("  g(r) > 1 : Particles cluster at this distance");
    println!("  g(r) = 1 : Random distribution");
    println!("  g(r) < 1 : Particles avoid this distance");
    println!();
    println!("{:>8} {:>12}", "r (um)", "g(r)");
    println!("{}", "-".repeat(22));

    for (r, g) in radii.iter().zip(g_r.iter()) {
        let indicator = if *g > 1.2 {
            " *** aggregation"
        } else if *g < 0.8 {
            " --- depletion"
        } else {
            ""
        };
        println!("{:8.2} {:12.4}{}", r, g, indicator);
    }
}
