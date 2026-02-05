//! Global parallel processing settings for OpenCASCADE operations.
//!
//! OCCT supports parallel processing for several operations using its built-in
//! thread pool (`OSD_ThreadPool`). This module provides functions to enable
//! parallel processing globally for better performance on multi-core systems.
//!
//! # Example
//!
//! ```ignore
//! use opencascade::parallel;
//!
//! // Enable all parallel processing at application startup
//! parallel::enable_all();
//!
//! // Or enable specific operations
//! parallel::set_mesh_parallel(true);
//! ```
//!
//! # Affected Operations
//!
//! - **Meshing** (`BRepMesh_IncrementalMesh`): Triangulation for visualization/export
//! - **Distance computation** (`BRepExtrema_DistShapeShape`): Collision detection
//!   (Note: This is enabled by default in our bindings)

use opencascade_sys::ffi;

/// Enable parallel processing for meshing operations.
///
/// When enabled, `BRepMesh_IncrementalMesh` will use multiple threads
/// to triangulate faces in parallel. This can significantly speed up
/// GLB/mesh export for complex shapes.
///
/// Default: `false`
pub fn set_mesh_parallel(enabled: bool) {
    ffi::BRepMesh_SetParallelDefault(enabled);
}

/// Check if parallel meshing is enabled.
pub fn is_mesh_parallel() -> bool {
    ffi::BRepMesh_IsParallelDefault()
}

/// Enable parallel processing for all supported operations.
///
/// This is a convenience function that enables:
/// - Parallel meshing
///
/// Note: Distance computation (`BRepExtrema_DistShapeShape`) is already
/// configured to use parallel processing by default in our bindings.
///
/// Call this once at application startup for best performance.
pub fn enable_all() {
    set_mesh_parallel(true);
}

/// Disable parallel processing for all operations.
///
/// This may be useful in scenarios where you're already running
/// OCCT operations from multiple threads and want to avoid
/// thread pool contention.
pub fn disable_all() {
    set_mesh_parallel(false);
}
