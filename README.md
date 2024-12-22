# RBC Aggregation Analysis

High-performance analysis pipeline for red blood cell aggregation from 3D confocal microscopy data. Written in Rust for memory safety and parallel processing of large datasets.

## Overview

Red blood cell aggregation is a critical parameter in hemorheology. Abnormal aggregation patterns can indicate various pathological conditions including cardiovascular disease, diabetes, and inflammatory states. This toolkit provides quantitative analysis of RBC spatial distributions from segmented confocal microscopy images.

### Pipeline Stages

```
TIFF Stack → Intensity QC → Segmentation* → Mesh Processing → Statistical Analysis
                                ↓
                        [External: ilastik/Fiji]
```

*Segmentation performed externally using machine learning tools (ilastik, Trainable Weka Segmentation)

## Features

- OBJ mesh loading with automatic triangulation
- Inertia tensor computation for shape classification
- Watertight mesh validation
- 3D pair correlation function g(r) with parallel computation
- Volume distribution statistics
- TIFF stack intensity profiling

## Installation

```bash
cargo build --release
```

## Usage

### Process Mesh Files

Extract geometric properties from segmented cell meshes:

```bash
rbc-analyze process \
    --input ./data/cells/ \
    --output results.csv \
    --min-volume 30 \
    --max-volume 200
```

Output CSV columns:
| Column | Description |
|--------|-------------|
| id | Cell identifier |
| x, y, z | Centroid position (um) |
| lambda1, lambda2, lambda3 | Principal moments of inertia |
| asphericity | Shape descriptor (0 = sphere) |
| acylindricity | Deviation from cylindrical symmetry |
| volume | Cell volume (um^3) |

### Pair Correlation Analysis

Compute radial distribution function to quantify aggregation:

```bash
rbc-analyze correlation \
    --input results.csv \
    --r-max 15.0 \
    --dr 0.25 \
    --domain-size 120
```

Interpretation:
- g(r) > 1 at contact distance indicates aggregation
- Peak height correlates with aggregation strength
- Peak position indicates rouleaux vs. cluster formation

### Intensity Profiling

Analyze Z-stack quality and identify optimal focal region:

```bash
rbc-analyze intensity \
    --input stack.tif \
    --start 20 \
    --end 80
```

## Architecture

```
src/
├── mesh/
│   ├── geometry.rs    # Mesh representation, volume, centroid
│   ├── inertia.rs     # Inertia tensor, shape descriptors  
│   └── filter.rs      # Validation criteria for valid RBCs
├── analysis/
│   ├── pair_correlation.rs   # g(r) computation
│   └── volume_distribution.rs # Statistical analysis
└── io/
    ├── obj_loader.rs  # Wavefront OBJ parser
    └── tiff_stack.rs  # Multi-page TIFF reader
```

## Shape Classification

RBC morphology is characterized by the eigenvalues of the gyration tensor:

| Morphology | Asphericity | Acylindricity | Clinical Relevance |
|------------|-------------|---------------|-------------------|
| Discocyte | High | Low | Normal healthy RBC |
| Spherocyte | Low | Low | Hereditary spherocytosis |
| Elliptocyte | High | High | Hereditary elliptocytosis |
| Stomatocyte | Medium | Medium | Liver disease, alcoholism |

## Performance

Parallel processing using Rayon enables efficient analysis of large datasets:

| Operation | 1000 cells | 10000 cells |
|-----------|-----------|-------------|
| Mesh loading | ~2s | ~18s |
| Property extraction | ~0.5s | ~4s |
| Pair correlation | ~1s | ~8s |

Benchmarks on AMD Ryzen 7 5800X, 32GB RAM.

## References

1. Baskurt, O.K., & Meiselman, H.J. (2003). Blood rheology and hemodynamics. Seminars in Thrombosis and Hemostasis.
2. Hansen, J.P., & McDonald, I.R. (2013). Theory of Simple Liquids. Academic Press.
3. Thevenaz, P., & Bhargava, S. (2017). Cell segmentation using ilastik. Nature Methods.

## License

MIT
