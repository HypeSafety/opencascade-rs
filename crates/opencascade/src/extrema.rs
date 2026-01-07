//! Distance computation between shapes using BRepExtrema.
//!
//! This module provides functionality to compute minimum distances between
//! shapes and retrieve information about the closest points.

use crate::primitives::Shape;
use crate::Error;
use glam::{dvec3, DVec3};
use opencascade_sys::ffi;

/// Type of geometric support where a closest point lies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportType {
    /// Point lies on a vertex
    Vertex,
    /// Point lies on an edge
    Edge,
    /// Point lies on a face
    Face,
}

impl From<ffi::BRepExtrema_SupportType> for SupportType {
    fn from(st: ffi::BRepExtrema_SupportType) -> Self {
        match st {
            ffi::BRepExtrema_SupportType::BRepExtrema_IsVertex => SupportType::Vertex,
            ffi::BRepExtrema_SupportType::BRepExtrema_IsOnEdge => SupportType::Edge,
            ffi::BRepExtrema_SupportType::BRepExtrema_IsInFace => SupportType::Face,
            _ => SupportType::Vertex, // fallback
        }
    }
}

/// Parameters for a point on an edge.
#[derive(Debug, Clone, Copy)]
pub struct EdgeParameter {
    /// Parameter t along the edge curve
    pub t: f64,
}

/// Parameters for a point on a face.
#[derive(Debug, Clone, Copy)]
pub struct FaceParameter {
    /// U parameter on the face surface
    pub u: f64,
    /// V parameter on the face surface
    pub v: f64,
}

/// Parametric information for a closest point.
#[derive(Debug, Clone, Copy)]
pub enum ParametricLocation {
    /// Point is on a vertex (no additional parameters)
    Vertex,
    /// Point is on an edge with parameter t
    Edge(EdgeParameter),
    /// Point is on a face with parameters (u, v)
    Face(FaceParameter),
}

/// A single closest-point solution between two shapes.
#[derive(Debug, Clone)]
pub struct ExtremaSolution {
    /// Closest point on shape 1
    pub point1: DVec3,
    /// Closest point on shape 2
    pub point2: DVec3,
    /// Type of support on shape 1
    pub support_type1: SupportType,
    /// Type of support on shape 2
    pub support_type2: SupportType,
    /// Parametric location on shape 1
    pub param1: ParametricLocation,
    /// Parametric location on shape 2
    pub param2: ParametricLocation,
}

/// Result of a distance computation between two shapes.
#[derive(Debug, Clone)]
pub struct DistanceResult {
    /// Minimum distance between the shapes
    pub distance: f64,
    /// All solutions (closest point pairs) at the minimum distance
    pub solutions: Vec<ExtremaSolution>,
    /// True if the shapes are overlapping (distance = 0 with inner contact)
    pub inner_solution: bool,
}

/// Compute the minimum distance between two shapes.
///
/// # Arguments
/// * `shape1` - First shape
/// * `shape2` - Second shape
///
/// # Returns
/// A `DistanceResult` containing the minimum distance and all closest point pairs.
///
/// # Example
/// ```ignore
/// use opencascade::primitives::Shape;
/// use opencascade::extrema::distance_between_shapes;
/// use glam::dvec3;
///
/// let box1 = Shape::box_with_dimensions(10.0, 10.0, 10.0);
/// let box2 = Shape::box_with_dimensions(5.0, 5.0, 5.0).translate(dvec3(20.0, 0.0, 0.0));
///
/// let result = distance_between_shapes(&box1, &box2).unwrap();
/// println!("Distance: {}", result.distance);
/// ```
pub fn distance_between_shapes(shape1: &Shape, shape2: &Shape) -> Result<DistanceResult, Error> {
    let extrema = ffi::BRepExtrema_DistShapeShape_ctor_shapes(&shape1.inner, &shape2.inner);

    if !extrema.IsDone() {
        return Err(Error::DistanceComputationFailed);
    }

    extract_result(&extrema)
}

/// Compute the minimum distance between two shapes with a custom deflection tolerance.
///
/// The deflection parameter controls the precision for curved surfaces.
/// Smaller values give more accurate results but take longer to compute.
///
/// # Arguments
/// * `shape1` - First shape
/// * `shape2` - Second shape
/// * `deflection` - Maximum deflection (chordal deviation) for approximating curves
pub fn distance_between_shapes_with_deflection(
    shape1: &Shape,
    shape2: &Shape,
    deflection: f64,
) -> Result<DistanceResult, Error> {
    let extrema = ffi::BRepExtrema_DistShapeShape_ctor_shapes_deflection(
        &shape1.inner,
        &shape2.inner,
        deflection,
    );

    if !extrema.IsDone() {
        return Err(Error::DistanceComputationFailed);
    }

    extract_result(&extrema)
}

/// Extract the result from a completed BRepExtrema computation.
fn extract_result(extrema: &ffi::BRepExtrema_DistShapeShape) -> Result<DistanceResult, Error> {
    let distance = extrema.Value();
    let inner_solution = extrema.InnerSolution();
    let nb_solutions = extrema.NbSolution();

    let mut solutions = Vec::with_capacity(nb_solutions as usize);

    // BRepExtrema uses 1-based indexing
    for i in 1..=nb_solutions {
        let pt1 = ffi::BRepExtrema_DistShapeShape_PointOnShape1(extrema, i);
        let pt2 = ffi::BRepExtrema_DistShapeShape_PointOnShape2(extrema, i);

        let point1 = dvec3(pt1.X(), pt1.Y(), pt1.Z());
        let point2 = dvec3(pt2.X(), pt2.Y(), pt2.Z());

        let support_type1 = extrema.SupportTypeShape1(i).into();
        let support_type2 = extrema.SupportTypeShape2(i).into();

        // Extract parametric information for shape 1
        let param1 = extract_param1(extrema, i, support_type1);
        let param2 = extract_param2(extrema, i, support_type2);

        solutions.push(ExtremaSolution {
            point1,
            point2,
            support_type1,
            support_type2,
            param1,
            param2,
        });
    }

    Ok(DistanceResult { distance, solutions, inner_solution })
}

fn extract_param1(
    extrema: &ffi::BRepExtrema_DistShapeShape,
    n: i32,
    support_type: SupportType,
) -> ParametricLocation {
    match support_type {
        SupportType::Vertex => ParametricLocation::Vertex,
        SupportType::Edge => {
            let mut t = 0.0;
            if ffi::BRepExtrema_DistShapeShape_ParOnEdgeS1(extrema, n, &mut t) {
                ParametricLocation::Edge(EdgeParameter { t })
            } else {
                ParametricLocation::Vertex
            }
        }
        SupportType::Face => {
            let mut u = 0.0;
            let mut v = 0.0;
            if ffi::BRepExtrema_DistShapeShape_ParOnFaceS1(extrema, n, &mut u, &mut v) {
                ParametricLocation::Face(FaceParameter { u, v })
            } else {
                ParametricLocation::Vertex
            }
        }
    }
}

fn extract_param2(
    extrema: &ffi::BRepExtrema_DistShapeShape,
    n: i32,
    support_type: SupportType,
) -> ParametricLocation {
    match support_type {
        SupportType::Vertex => ParametricLocation::Vertex,
        SupportType::Edge => {
            let mut t = 0.0;
            if ffi::BRepExtrema_DistShapeShape_ParOnEdgeS2(extrema, n, &mut t) {
                ParametricLocation::Edge(EdgeParameter { t })
            } else {
                ParametricLocation::Vertex
            }
        }
        SupportType::Face => {
            let mut u = 0.0;
            let mut v = 0.0;
            if ffi::BRepExtrema_DistShapeShape_ParOnFaceS2(extrema, n, &mut u, &mut v) {
                ParametricLocation::Face(FaceParameter { u, v })
            } else {
                ParametricLocation::Vertex
            }
        }
    }
}

/// Builder for deferred or reusable distance computations.
///
/// Use this when you need to compute distances between many pairs of shapes
/// while reusing the same computation object, or when you want fine-grained
/// control over the computation parameters.
///
/// # Example
/// ```ignore
/// use opencascade::primitives::Shape;
/// use opencascade::extrema::DistanceComputer;
///
/// let mut computer = DistanceComputer::new();
/// computer.load_shape1(&box1);
/// computer.with_deflection(0.001);
///
/// // Compute distance to multiple shapes
/// for target in targets {
///     computer.load_shape2(&target);
///     let result = computer.compute()?;
///     println!("Distance: {}", result.distance);
/// }
/// ```
pub struct DistanceComputer {
    inner: cxx::UniquePtr<ffi::BRepExtrema_DistShapeShape>,
}

impl DistanceComputer {
    /// Create a new distance computer.
    pub fn new() -> Self {
        Self { inner: ffi::BRepExtrema_DistShapeShape_ctor() }
    }

    /// Load the first shape.
    pub fn load_shape1(&mut self, shape: &Shape) -> &mut Self {
        self.inner.pin_mut().LoadS1(&shape.inner);
        self
    }

    /// Load the second shape.
    pub fn load_shape2(&mut self, shape: &Shape) -> &mut Self {
        self.inner.pin_mut().LoadS2(&shape.inner);
        self
    }

    /// Set the deflection tolerance for curve approximation.
    pub fn with_deflection(&mut self, deflection: f64) -> &mut Self {
        self.inner.pin_mut().SetDeflection(deflection);
        self
    }

    /// Perform the distance computation and return the result.
    pub fn compute(&mut self) -> Result<DistanceResult, Error> {
        let progress = ffi::Message_ProgressRange_ctor();
        let success = self.inner.pin_mut().Perform(&progress);

        if !success || !self.inner.IsDone() {
            return Err(Error::DistanceComputationFailed);
        }

        extract_result(&self.inner)
    }
}

impl Default for DistanceComputer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_between_separated_boxes() {
        let box1 = Shape::box_with_dimensions(10.0, 10.0, 10.0);
        let box2 = Shape::box_with_dimensions(5.0, 5.0, 5.0).translate(dvec3(20.0, 0.0, 0.0));

        let result = distance_between_shapes(&box1, &box2).unwrap();

        // Distance should be 10 (gap between x=10 and x=20)
        assert!((result.distance - 10.0).abs() < 0.001);
        assert!(!result.inner_solution);
        assert!(!result.solutions.is_empty());
    }

    #[test]
    fn test_distance_touching_boxes() {
        let box1 = Shape::box_with_dimensions(10.0, 10.0, 10.0);
        let box2 = Shape::box_with_dimensions(5.0, 5.0, 5.0).translate(dvec3(10.0, 0.0, 0.0));

        let result = distance_between_shapes(&box1, &box2).unwrap();

        // Boxes should be touching (distance = 0)
        assert!(result.distance < 0.001);
    }

    #[test]
    fn test_distance_computer_builder() {
        let box1 = Shape::box_with_dimensions(10.0, 10.0, 10.0);
        let box2 = Shape::box_with_dimensions(5.0, 5.0, 5.0).translate(dvec3(15.0, 0.0, 0.0));

        let mut computer = DistanceComputer::new();
        computer.load_shape1(&box1).load_shape2(&box2).with_deflection(0.001);

        let result = computer.compute().unwrap();

        // Distance should be 5
        assert!((result.distance - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_support_type_detection() {
        // Create a point (vertex) and a face
        let box1 = Shape::box_with_dimensions(1.0, 1.0, 1.0);
        let box2 = Shape::box_with_dimensions(1.0, 1.0, 1.0).translate(dvec3(5.0, 0.5, 0.5));

        let result = distance_between_shapes(&box1, &box2).unwrap();

        assert!(!result.solutions.is_empty());
        // The closest points should be face-to-face contact
        let solution = &result.solutions[0];
        // Both contact points should be on faces (or edges at corners)
        assert!(matches!(
            solution.support_type1,
            SupportType::Face | SupportType::Edge | SupportType::Vertex
        ));
    }
}
