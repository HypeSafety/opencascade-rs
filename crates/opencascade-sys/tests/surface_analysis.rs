//! Surface analysis tests for sheet metal geometry.
//!
//! Note: Run with `--test-threads=1` as OCCT is not thread-safe.

use opencascade_sys::ffi::{
    BRepAdaptor_Surface_Cylinder, BRepAdaptor_Surface_Plane, BRepAdaptor_Surface_ctor,
    GeomAbs_SurfaceType, Message_ProgressRange_ctor, STEPControl_Reader_ctor, TopAbs_ShapeEnum,
    TopExp_Explorer_ctor, TopoDS_cast_to_face, gp_Ax3_Direction, read_step,
};

fn sample_path(name: &str) -> String {
    let home = std::env::var("HOME").expect("HOME not set");
    format!("{}/Dev/Worktrees/labyrinth/sample_data/{}", home, name)
}

#[test]
fn test_detect_surface_types() {
    // Load STEP file
    let mut reader = STEPControl_Reader_ctor();
    let status = read_step(reader.pin_mut(), sample_path("splice.step"));
    assert_eq!(status, opencascade_sys::ffi::IFSelect_ReturnStatus::IFSelect_RetDone);

    let progress = Message_ProgressRange_ctor();
    let num_roots = reader.pin_mut().TransferRoots(&progress);
    assert!(num_roots > 0, "Should transfer at least one root");

    let shape = opencascade_sys::ffi::one_shape_step(&reader);
    assert!(!shape.IsNull(), "Shape should not be null");

    // Count surface types
    let mut plane_count = 0;
    let mut cylinder_count = 0;
    let mut other_count = 0;

    let mut face_explorer = TopExp_Explorer_ctor(&shape, TopAbs_ShapeEnum::TopAbs_FACE);
    while face_explorer.More() {
        let face = TopoDS_cast_to_face(face_explorer.Current());
        let surface = BRepAdaptor_Surface_ctor(face, true);

        match surface.GetType() {
            GeomAbs_SurfaceType::GeomAbs_Plane => plane_count += 1,
            GeomAbs_SurfaceType::GeomAbs_Cylinder => cylinder_count += 1,
            _ => other_count += 1,
        }

        face_explorer.pin_mut().Next();
    }

    // Sheet metal parts should have both planar faces and cylindrical bends
    assert!(plane_count > 0, "Should have planar faces, got {}", plane_count);
    assert!(cylinder_count > 0, "Should have cylindrical faces (bends), got {}", cylinder_count);
    println!(
        "Surface types: {} planes, {} cylinders, {} other",
        plane_count, cylinder_count, other_count
    );
}

#[test]
fn test_cylinder_radius() {
    // Load STEP file
    let mut reader = STEPControl_Reader_ctor();
    let status = read_step(reader.pin_mut(), sample_path("splice.step"));
    assert_eq!(status, opencascade_sys::ffi::IFSelect_ReturnStatus::IFSelect_RetDone);

    let progress = Message_ProgressRange_ctor();
    reader.pin_mut().TransferRoots(&progress);
    let shape = opencascade_sys::ffi::one_shape_step(&reader);

    // Find a cylindrical face and check its radius
    let mut face_explorer = TopExp_Explorer_ctor(&shape, TopAbs_ShapeEnum::TopAbs_FACE);
    let mut found_cylinder = false;

    while face_explorer.More() {
        let face = TopoDS_cast_to_face(face_explorer.Current());
        let surface = BRepAdaptor_Surface_ctor(face, true);

        if surface.GetType() == GeomAbs_SurfaceType::GeomAbs_Cylinder {
            let cylinder = BRepAdaptor_Surface_Cylinder(&surface);
            let radius = cylinder.Radius();

            // Radius should be positive and reasonable for sheet metal (typically 0.5mm - 50mm)
            assert!(radius > 0.0, "Cylinder radius should be positive");
            println!("Found cylinder with radius: {:.3} mm", radius);
            found_cylinder = true;
            break;
        }

        face_explorer.pin_mut().Next();
    }

    assert!(found_cylinder, "Should find at least one cylindrical face");
}

#[test]
fn test_plane_normal() {
    // Load STEP file
    let mut reader = STEPControl_Reader_ctor();
    let status = read_step(reader.pin_mut(), sample_path("splice.step"));
    assert_eq!(status, opencascade_sys::ffi::IFSelect_ReturnStatus::IFSelect_RetDone);

    let progress = Message_ProgressRange_ctor();
    reader.pin_mut().TransferRoots(&progress);
    let shape = opencascade_sys::ffi::one_shape_step(&reader);

    // Find a planar face and check its normal
    let mut face_explorer = TopExp_Explorer_ctor(&shape, TopAbs_ShapeEnum::TopAbs_FACE);
    let mut found_plane = false;

    while face_explorer.More() {
        let face = TopoDS_cast_to_face(face_explorer.Current());
        let surface = BRepAdaptor_Surface_ctor(face, true);

        if surface.GetType() == GeomAbs_SurfaceType::GeomAbs_Plane {
            let plane = BRepAdaptor_Surface_Plane(&surface);
            let position = opencascade_sys::ffi::gp_Pln_Position(&plane);
            let normal = gp_Ax3_Direction(&position);

            // Normal should be a unit vector
            let length_sq = normal.X().powi(2) + normal.Y().powi(2) + normal.Z().powi(2);
            assert!(
                (length_sq - 1.0).abs() < 1e-6,
                "Normal should be unit vector, got length {}",
                length_sq.sqrt()
            );

            println!(
                "Found plane with normal: ({:.3}, {:.3}, {:.3})",
                normal.X(),
                normal.Y(),
                normal.Z()
            );
            found_plane = true;
            break;
        }

        face_explorer.pin_mut().Next();
    }

    assert!(found_plane, "Should find at least one planar face");
}
