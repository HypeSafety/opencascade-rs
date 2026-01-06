╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌
 BRepExtrema Bindings Implementation Plan

 Overview

 Add BRepExtrema_DistShapeShape bindings to opencascade-rs for computing minimum distances between
 shapes, including contact points and parametric information.

 Files to Modify

 | File                                       | Action | Description                            |
 |--------------------------------------------|--------|----------------------------------------|
 | crates/opencascade-sys/include/wrapper.hxx | Modify | Add includes and C++ wrapper functions |
 | crates/opencascade-sys/src/lib.rs          | Modify | Add FFI type and function declarations |
 | crates/opencascade/src/extrema.rs          | Create | New high-level extrema module          |
 | crates/opencascade/src/lib.rs              | Modify | Add module export and error variant    |
 | crates/opencascade/src/primitives/shape.rs | Modify | Add distance_to() convenience method   |

 Implementation Steps

 Step 1: C++ Wrapper (wrapper.hxx)

 Add include at top:
 #include <BRepExtrema_DistShapeShape.hxx>

 Add wrapper functions (after existing wrappers ~line 500):

- BRepExtrema_DistShapeShape_ctor() - empty constructor
- BRepExtrema_DistShapeShape_ctor_shapes(shape1, shape2) - compute on construction
- BRepExtrema_DistShapeShape_ctor_shapes_deflection(shape1, shape2, deflection) - with tolerance
- BRepExtrema_DistShapeShape_PointOnShape1(extrema, n) - returns UniquePtr<gp_Pnt>
- BRepExtrema_DistShapeShape_PointOnShape2(extrema, n) - returns UniquePtr<gp_Pnt>
- BRepExtrema_DistShapeShape_SupportOnShape1(extrema, n) - returns UniquePtr<TopoDS_Shape>
- BRepExtrema_DistShapeShape_SupportOnShape2(extrema, n) - returns UniquePtr<TopoDS_Shape>
- BRepExtrema_DistShapeShape_ParOnEdgeS1/S2(extrema, n, &t) - edge parameter
- BRepExtrema_DistShapeShape_ParOnFaceS1/S2(extrema, n, &u, &v) - face UV parameters

 Step 2: FFI Bindings (opencascade-sys/src/lib.rs)

 Add enum (in enums section ~line 80):
 #[repr(u32)]
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 pub enum BRepExtrema_SupportType {
     BRepExtrema_IsVertex,
     BRepExtrema_IsOnEdge,
     BRepExtrema_IsInFace,
 }

 Add to unsafe extern "C++" block:
 type BRepExtrema_DistShapeShape;

 // Constructors
 pub fn BRepExtrema_DistShapeShape_ctor() -> UniquePtr<BRepExtrema_DistShapeShape>;
 pub fn BRepExtrema_DistShapeShape_ctor_shapes(shape1: &TopoDS_Shape, shape2: &TopoDS_Shape) ->
 UniquePtr<BRepExtrema_DistShapeShape>;
 pub fn BRepExtrema_DistShapeShape_ctor_shapes_deflection(shape1: &TopoDS_Shape, shape2:
 &TopoDS_Shape, deflection: f64) -> UniquePtr<BRepExtrema_DistShapeShape>;

 // Core methods (direct binding)
 pub fn Perform(self: Pin<&mut BRepExtrema_DistShapeShape>) -> bool;
 pub fn LoadS1(self: Pin<&mut BRepExtrema_DistShapeShape>, shape: &TopoDS_Shape);
 pub fn LoadS2(self: Pin<&mut BRepExtrema_DistShapeShape>, shape: &TopoDS_Shape);
 pub fn SetDeflection(self: Pin<&mut BRepExtrema_DistShapeShape>, deflection: f64);
 pub fn IsDone(self: &BRepExtrema_DistShapeShape) -> bool;
 pub fn NbSolution(self: &BRepExtrema_DistShapeShape) -> i32;
 pub fn Value(self: &BRepExtrema_DistShapeShape) -> f64;
 pub fn InnerSolution(self: &BRepExtrema_DistShapeShape) -> bool;
 pub fn SupportTypeShape1(self: &BRepExtrema_DistShapeShape, n: i32) -> BRepExtrema_SupportType;
 pub fn SupportTypeShape2(self: &BRepExtrema_DistShapeShape, n: i32) -> BRepExtrema_SupportType;

 // Wrapper functions (declared in wrapper.hxx)
 pub fn BRepExtrema_DistShapeShape_PointOnShape1(extrema: &BRepExtrema_DistShapeShape, n: i32) ->
 UniquePtr<gp_Pnt>;
 pub fn BRepExtrema_DistShapeShape_PointOnShape2(extrema: &BRepExtrema_DistShapeShape, n: i32) ->
 UniquePtr<gp_Pnt>;
 pub fn BRepExtrema_DistShapeShape_SupportOnShape1(extrema: &BRepExtrema_DistShapeShape, n: i32) ->
 UniquePtr<TopoDS_Shape>;
 pub fn BRepExtrema_DistShapeShape_SupportOnShape2(extrema: &BRepExtrema_DistShapeShape, n: i32) ->
 UniquePtr<TopoDS_Shape>;
 pub fn BRepExtrema_DistShapeShape_ParOnEdgeS1(extrema: &BRepExtrema_DistShapeShape, n: i32, t: &mut
  f64) -> bool;
 pub fn BRepExtrema_DistShapeShape_ParOnEdgeS2(extrema: &BRepExtrema_DistShapeShape, n: i32, t: &mut
  f64) -> bool;
 pub fn BRepExtrema_DistShapeShape_ParOnFaceS1(extrema: &BRepExtrema_DistShapeShape, n: i32, u: &mut
  f64, v: &mut f64) -> bool;
 pub fn BRepExtrema_DistShapeShape_ParOnFaceS2(extrema: &BRepExtrema_DistShapeShape, n: i32, u: &mut
  f64, v: &mut f64) -> bool;

 Step 3: High-Level API (opencascade/src/extrema.rs)

 Create new module with:

 Types:

- SupportType enum (Vertex, Edge, Face)
- ExtremaSolution struct with point1, point2, support types, optional parameters
- DistanceResult struct with distance, solutions vec, inner_solution flag

 Functions:

- distance_between_shapes(shape1, shape2) -> Result<DistanceResult, Error>
- distance_between_shapes_with_deflection(shape1, shape2, deflection) -> Result<DistanceResult,
 Error>

 Builder:

- DistanceComputer for deferred/reusable computation with .load_shape1(), .load_shape2(),
 .with_deflection(), .compute()

 Step 4: Update lib.rs and Shape

 opencascade/src/lib.rs:

- Add pub mod extrema;
- Add error variant: #[error("distance computation failed")] DistanceComputationFailed

 opencascade/src/primitives/shape.rs:

- Add convenience method: pub fn distance_to(&self, other: &Shape) -> Result<DistanceResult, Error>

 Build System

 No changes needed - TKTopAlgo (containing BRepExtrema) is already linked in build.rs.

 API Usage Example

 use opencascade::primitives::Shape;
 use glam::dvec3;

 let box1 = Shape::box_with_dimensions(10.0, 10.0, 10.0);
 let box2 = Shape::box_with_dimensions(5.0, 5.0, 5.0).translate(dvec3(20.0, 0.0, 0.0));

 let result = box1.distance_to(&box2)?;
 println!("Distance: {}", result.distance);  // 10.0
 println!("Contact point on box1: {:?}", result.solutions[0].point1);
 println!("Contact point on box2: {:?}", result.solutions[0].point2);

 Testing

 Add inline tests in extrema.rs module testing:

- Distance between separated boxes
- Touching shapes (zero distance)
- Support type detection (vertex, edge, face contact)
- Builder pattern usage
