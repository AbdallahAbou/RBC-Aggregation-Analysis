use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::PathBuf;
use tracing::{info, warn};

mod mesh;
mod analysis;
mod io;

use mesh::{Mesh, MeshFilter};
use analysis::PairCorrelation;
use io::ObjLoader;

#[derive(Parser)]
#[command(name = "rbc-analyze")]
#[command(about = "Analyze RBC aggregation from 3D microscopy data", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process mesh files and extract cell properties
    Process {
        /// Input directory containing .obj files
        #[arg(short, long)]
        input: PathBuf,

        /// Output CSV file for results
        #[arg(short, long)]
        output: PathBuf,

        /// Minimum valid cell volume (um^3)
        #[arg(long, default_value = "30")]
        min_volume: f64,

        /// Maximum valid cell volume (um^3)
        #[arg(long, default_value = "200")]
        max_volume: f64,
    },

    /// Compute pair correlation function g(r)
    Correlation {
        /// Input CSV with cell positions
        #[arg(short, long)]
        input: PathBuf,

        /// Maximum correlation distance
        #[arg(long, default_value = "10.0")]
        r_max: f64,

        /// Radial bin width
        #[arg(long, default_value = "0.25")]
        dr: f64,

        /// Domain size (cube side length)
        #[arg(long, default_value = "120.0")]
        domain_size: f64,
    },

    /// Analyze intensity profile from TIFF stack
    Intensity {
        /// Input TIFF file
        #[arg(short, long)]
        input: PathBuf,

        /// Start slice index
        #[arg(long)]
        start: Option<usize>,

        /// End slice index
        #[arg(long)]
        end: Option<usize>,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Process { input, output, min_volume, max_volume } => {
            process_meshes(&input, &output, min_volume, max_volume)?;
        }
        Commands::Correlation { input, r_max, dr, domain_size } => {
            compute_correlation(&input, r_max, dr, domain_size)?;
        }
        Commands::Intensity { input, start, end } => {
            analyze_intensity(&input, start, end)?;
        }
    }

    Ok(())
}

fn process_meshes(
    input_dir: &PathBuf,
    output: &PathBuf,
    min_volume: f64,
    max_volume: f64,
) -> Result<()> {
    let obj_files: Vec<_> = std::fs::read_dir(input_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "obj"))
        .collect();

    let pb = ProgressBar::new(obj_files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
    );

    let filter = MeshFilter::new()
        .with_volume_bounds(min_volume, max_volume)
        .require_watertight()
        .require_positive_centroid();

    let results: Vec<_> = obj_files
        .par_iter()
        .filter_map(|entry| {
            pb.inc(1);
            let path = entry.path();
            
            match ObjLoader::load(&path) {
                Ok(mesh) => {
                    if filter.accepts(&mesh) {
                        Some(mesh.to_cell_record())
                    } else {
                        None
                    }
                }
                Err(e) => {
                    warn!("Failed to load {:?}: {}", path, e);
                    None
                }
            }
        })
        .collect();

    pb.finish_with_message("Processing complete");

    info!("Processed {} cells, {} passed filters", obj_files.len(), results.len());

    // Write results to CSV
    let mut writer = csv::Writer::from_path(output)?;
    writer.write_record(&[
        "id", "x", "y", "z", 
        "lambda1", "lambda2", "lambda3",
        "asphericity", "acylindricity", "volume"
    ])?;

    for record in results {
        writer.write_record(&[
            record.id,
            record.x.to_string(),
            record.y.to_string(),
            record.z.to_string(),
            record.lambda1.to_string(),
            record.lambda2.to_string(),
            record.lambda3.to_string(),
            record.asphericity.to_string(),
            record.acylindricity.to_string(),
            record.volume.to_string(),
        ])?;
    }

    writer.flush()?;
    info!("Results written to {:?}", output);

    Ok(())
}

fn compute_correlation(
    input: &PathBuf,
    r_max: f64,
    dr: f64,
    domain_size: f64,
) -> Result<()> {
    let mut reader = csv::Reader::from_path(input)?;
    let mut positions = Vec::new();

    for result in reader.records() {
        let record = result?;
        let x: f64 = record[1].parse()?;
        let y: f64 = record[2].parse()?;
        let z: f64 = record[3].parse()?;
        positions.push([x, y, z]);
    }

    info!("Loaded {} cell positions", positions.len());

    let pcf = PairCorrelation::new(domain_size, r_max, dr);
    let (g_r, radii) = pcf.compute(&positions);

    println!("\nPair Correlation Function g(r):");
    println!("{:>8} {:>12}", "r", "g(r)");
    println!("{}", "-".repeat(22));
    
    for (r, g) in radii.iter().zip(g_r.iter()) {
        println!("{:8.3} {:12.6}", r, g);
    }

    Ok(())
}

fn analyze_intensity(
    input: &PathBuf,
    start: Option<usize>,
    end: Option<usize>,
) -> Result<()> {
    use io::TiffStack;

    let stack = TiffStack::open(input)?;
    let n_slices = stack.num_slices();
    
    let start_idx = start.unwrap_or(0);
    let end_idx = end.unwrap_or(n_slices);

    info!("Analyzing slices {} to {} of {}", start_idx, end_idx, n_slices);

    let intensities = stack.compute_slice_intensities(start_idx, end_idx)?;
    let mean_intensity: f64 = intensities.iter().sum::<f64>() / intensities.len() as f64;

    println!("\nIntensity Profile:");
    println!("Slices: {} - {}", start_idx, end_idx);
    println!("Mean intensity: {:.2}", mean_intensity);
    println!("\n{:>6} {:>12}", "Slice", "Intensity");
    println!("{}", "-".repeat(20));

    for (i, intensity) in intensities.iter().enumerate() {
        println!("{:6} {:12.2}", start_idx + i, intensity);
    }

    Ok(())
}
