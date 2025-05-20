# RBC Aggregation Analysis

High-performance toolkit for analyzing red blood cell aggregation from 3D confocal microscopy data.

## Features

- **Mesh Processing**: Load and process 3D cell meshes from OBJ files
- **Geometric Analysis**: Volume, centroid, and inertia tensor computation
- **Pair Correlation**: Radial distribution function g(r) for spatial analysis
- **Volume Statistics**: Distribution analysis with histogram generation

## Installation

```bash
cargo build --release
```

## Usage

```bash
rbc-analyze --input ./meshes --output ./results --r-max 50.0
```

## API Reference

### Mesh

```rust
use rbc_aggregation::Mesh;

let mesh = ObjLoader::load("cell.obj")?;
let volume = mesh.compute_volume();
let centroid = mesh.compute_centroid();
```

### Analysis

```rust
use rbc_aggregation::{PairCorrelation, VolumeDistribution};

let mut pcf = PairCorrelation::new(50.0, 100);
pcf.compute(&centroids, box_volume);

let dist = VolumeDistribution::from_volumes(volumes);
```

## License

MIT
