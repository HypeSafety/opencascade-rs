use thiserror::Error;

pub mod angle;
pub mod bounding_box;
pub mod extrema;
pub mod kicad;
pub mod mesh;
pub mod primitives;
pub mod section;
pub mod workplane;

mod law_function;
mod make_pipe_shell;

/// Re-export the FFI module for direct access to OpenCascade bindings.
pub use opencascade_sys::ffi;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to write STL file")]
    StlWriteFailed,
    #[error("failed to read STEP file")]
    StepReadFailed,
    #[error("failed to read IGES file")]
    IgesReadFailed,
    #[error("failed to read KiCAD PCB file: {0}")]
    KicadReadFailed(#[from] kicad_parser::Error),
    #[error("failed to write STEP file")]
    StepWriteFailed,
    #[error("failed to write IGES file")]
    IgesWriteFailed,
    #[error("failed to triangulate Shape")]
    TriangulationFailed,
    #[error("encountered a face with no triangulation")]
    UntriangulatedFace,
    #[error("at least 2 points are required for creating a wire")]
    NotEnoughPoints,
    #[error("loft operation failed")]
    LoftFailed,
    #[error("distance computation failed")]
    DistanceComputationFailed,
    #[error("shape offset operation failed")]
    OffsetFailed,
}
