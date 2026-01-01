use crate::{
    angle::Angle,
    law_function::law_function_from_graph,
    make_pipe_shell::make_pipe_shell_with_law_function,
    primitives::{
        make_axis_1, make_point, make_vec, EdgeIterator, JoinType, Shape, Solid, Surface, Wire,
    },
    workplane::Workplane,
};
use cxx::UniquePtr;
use glam::{dvec3, DVec3};
use opencascade_sys::ffi;

pub struct Face {
    pub(crate) inner: UniquePtr<ffi::TopoDS_Face>,
}

impl AsRef<Face> for Face {
    fn as_ref(&self) -> &Face {
        self
    }
}

impl Face {
    pub(crate) fn from_face(face: &ffi::TopoDS_Face) -> Self {
        let inner = ffi::TopoDS_Face_to_owned(face);

        Self { inner }
    }

    /// Get a reference to the underlying OpenCascade face.
    pub fn inner(&self) -> &ffi::TopoDS_Face {
        &self.inner
    }

    fn from_make_face(make_face: UniquePtr<ffi::BRepBuilderAPI_MakeFace>) -> Self {
        Self::from_face(make_face.Face())
    }

    pub fn from_wire(wire: &Wire) -> Self {
        let only_plane = false;
        let make_face = ffi::BRepBuilderAPI_MakeFace_wire(&wire.inner, only_plane);

        Self::from_make_face(make_face)
    }

    pub fn from_surface(surface: &Surface) -> Self {
        const EDGE_TOLERANCE: f64 = 0.0001;

        let make_face = ffi::BRepBuilderAPI_MakeFace_surface(&surface.inner, EDGE_TOLERANCE);

        Self::from_make_face(make_face)
    }

    #[must_use]
    pub fn extrude(&self, dir: DVec3) -> Solid {
        let prism_vec = make_vec(dir);

        let copy = false;
        let canonize = true;

        let inner_shape = ffi::cast_face_to_shape(&self.inner);
        let mut make_solid =
            ffi::BRepPrimAPI_MakePrism_ctor(inner_shape, &prism_vec, copy, canonize);
        let extruded_shape = make_solid.pin_mut().Shape();
        let solid = ffi::TopoDS_cast_to_solid(extruded_shape);

        Solid::from_solid(solid)
    }

    #[must_use]
    pub fn extrude_to_face(&self, shape_with_face: &Shape, face: &Face) -> Shape {
        let profile_base = &self.inner;
        let sketch_base = ffi::TopoDS_Face_ctor();
        let angle = 0.0;
        let fuse = 1; // 0 = subtractive, 1 = additive
        let modify = false;

        let mut make_prism = ffi::BRepFeat_MakeDPrism_ctor(
            &shape_with_face.inner,
            profile_base,
            &sketch_base,
            angle,
            fuse,
            modify,
        );

        let until_face = ffi::cast_face_to_shape(&face.inner);
        make_prism.pin_mut().perform_until_face(until_face);

        Shape::from_shape(make_prism.pin_mut().Shape())
    }

    #[must_use]
    pub fn subtractive_extrude(&self, shape_with_face: &Shape, height: f64) -> Shape {
        let profile_base = &self.inner;
        let sketch_base = ffi::TopoDS_Face_ctor();
        let angle = 0.0;
        let fuse = 0; // 0 = subtractive, 1 = additive
        let modify = false;

        let mut make_prism = ffi::BRepFeat_MakeDPrism_ctor(
            &shape_with_face.inner,
            profile_base,
            &sketch_base,
            angle,
            fuse,
            modify,
        );

        make_prism.pin_mut().perform_with_height(height);

        Shape::from_shape(make_prism.pin_mut().Shape())
    }

    #[must_use]
    pub fn revolve(&self, origin: DVec3, axis: DVec3, angle: Option<Angle>) -> Solid {
        let revol_vec = make_axis_1(origin, axis);

        let angle = angle.map(Angle::radians).unwrap_or(std::f64::consts::PI * 2.0);
        let copy = false;

        let inner_shape = ffi::cast_face_to_shape(&self.inner);
        let mut make_solid = ffi::BRepPrimAPI_MakeRevol_ctor(inner_shape, &revol_vec, angle, copy);
        let revolved_shape = make_solid.pin_mut().Shape();
        let solid = ffi::TopoDS_cast_to_solid(revolved_shape);

        Solid::from_solid(solid)
    }

    /// Fillets the face edges by a given radius at each vertex
    #[must_use]
    pub fn fillet(&self, radius: f64) -> Self {
        let mut make_fillet = ffi::BRepFilletAPI_MakeFillet2d_ctor(&self.inner);

        let face_shape = ffi::cast_face_to_shape(&self.inner);

        // We use a shape map here to avoid duplicates.
        let mut shape_map = ffi::new_indexed_map_of_shape();
        ffi::map_shapes(face_shape, ffi::TopAbs_ShapeEnum::TopAbs_VERTEX, shape_map.pin_mut());

        for i in 1..=shape_map.Extent() {
            let vertex = ffi::TopoDS_cast_to_vertex(shape_map.FindKey(i));
            ffi::BRepFilletAPI_MakeFillet2d_add_fillet(make_fillet.pin_mut(), vertex, radius);
        }

        make_fillet.pin_mut().Build(&ffi::Message_ProgressRange_ctor());

        let result_shape = make_fillet.pin_mut().Shape();
        let result_face = ffi::TopoDS_cast_to_face(result_shape);

        Self::from_face(result_face)
    }

    /// Chamfer the wire edges at each vertex by a given distance
    #[must_use]
    pub fn chamfer(&self, distance_1: f64) -> Self {
        // TODO - Support asymmetric chamfers.
        let distance_2 = distance_1;

        let face_shape = ffi::cast_face_to_shape(&self.inner);

        let mut make_fillet = ffi::BRepFilletAPI_MakeFillet2d_ctor(&self.inner);

        let mut vertex_map = ffi::new_indexed_map_of_shape();
        ffi::map_shapes(face_shape, ffi::TopAbs_ShapeEnum::TopAbs_VERTEX, vertex_map.pin_mut());

        // Get map of vertices to edges so we can find the edges connected to each vertex.
        let mut data_map = ffi::new_indexed_data_map_of_shape_list_of_shape();
        ffi::map_shapes_and_ancestors(
            face_shape,
            ffi::TopAbs_ShapeEnum::TopAbs_VERTEX,
            ffi::TopAbs_ShapeEnum::TopAbs_EDGE,
            data_map.pin_mut(),
        );

        // Chamfer at vertex of all edges.
        for i in 1..=vertex_map.Extent() {
            let edges = ffi::shape_list_to_vector(data_map.FindFromIndex(i));
            let edge_1 = edges.get(0).expect("Vertex has no edges");
            let edge_2 = edges.get(1).expect("Vertex has only one edge");
            ffi::BRepFilletAPI_MakeFillet2d_add_chamfer(
                make_fillet.pin_mut(),
                ffi::TopoDS_cast_to_edge(edge_1),
                ffi::TopoDS_cast_to_edge(edge_2),
                distance_1,
                distance_2,
            );
        }

        let filleted_shape = make_fillet.pin_mut().Shape();
        let result_face = ffi::TopoDS_cast_to_face(filleted_shape);

        Self::from_face(result_face)
    }

    /// Offset the face by a given distance and join settings
    #[must_use]
    pub fn offset(&self, distance: f64, join_type: JoinType) -> Self {
        let mut make_offset =
            ffi::BRepOffsetAPI_MakeOffset_face_ctor(&self.inner, join_type.into());
        make_offset.pin_mut().Perform(distance, 0.0);

        let offset_shape = make_offset.pin_mut().Shape();
        let result_wire = ffi::TopoDS_cast_to_wire(offset_shape);
        let wire = Wire::from_wire(result_wire);

        wire.to_face()
    }

    /// Sweep the face along a path to produce a solid
    #[must_use]
    pub fn sweep_along(&self, path: &Wire) -> Solid {
        let profile_shape = ffi::cast_face_to_shape(&self.inner);
        let mut make_pipe = ffi::BRepOffsetAPI_MakePipe_ctor(&path.inner, profile_shape);

        let pipe_shape = make_pipe.pin_mut().Shape();
        let result_solid = ffi::TopoDS_cast_to_solid(pipe_shape);

        Solid::from_solid(result_solid)
    }

    /// Sweep the face along a path, modulated by a function, to produce a solid
    #[must_use]
    pub fn sweep_along_with_radius_values(
        &self,
        path: &Wire,
        radius_values: impl IntoIterator<Item = (f64, f64)>,
    ) -> Solid {
        let law_function = law_function_from_graph(radius_values);
        let law_handle = ffi::Law_Function_to_handle(law_function);

        let profile_wire = ffi::outer_wire(&self.inner);
        let mut make_pipe_shell =
            make_pipe_shell_with_law_function(&profile_wire, &path.inner, &law_handle);

        make_pipe_shell.pin_mut().Build(&ffi::Message_ProgressRange_ctor());
        make_pipe_shell.pin_mut().MakeSolid();
        let pipe_shape = make_pipe_shell.pin_mut().Shape();
        let result_solid = ffi::TopoDS_cast_to_solid(pipe_shape);

        Solid::from_solid(result_solid)
    }

    pub fn edges(&self) -> EdgeIterator {
        let explorer = ffi::TopExp_Explorer_ctor(
            ffi::cast_face_to_shape(&self.inner),
            ffi::TopAbs_ShapeEnum::TopAbs_EDGE,
        );

        EdgeIterator { explorer }
    }

    pub fn center_of_mass(&self) -> DVec3 {
        let mut props = ffi::GProp_GProps_ctor();

        let inner_shape = ffi::cast_face_to_shape(&self.inner);
        ffi::BRepGProp_SurfaceProperties(inner_shape, props.pin_mut());

        let center = ffi::GProp_GProps_CentreOfMass(&props);

        dvec3(center.X(), center.Y(), center.Z())
    }

    pub fn normal_at(&self, pos: DVec3) -> DVec3 {
        let surface = ffi::BRep_Tool_Surface(&self.inner);
        let projector = ffi::GeomAPI_ProjectPointOnSurf_ctor(&make_point(pos), &surface);
        let mut u: f64 = 0.0;
        let mut v: f64 = 0.0;

        projector.LowerDistanceParameters(&mut u, &mut v);

        let mut p = ffi::new_point(0.0, 0.0, 0.0);
        let mut normal = ffi::new_vec(0.0, 1.0, 0.0);

        let face = ffi::BRepGProp_Face_ctor(&self.inner);
        face.Normal(u, v, p.pin_mut(), normal.pin_mut());

        dvec3(normal.X(), normal.Y(), normal.Z())
    }

    pub fn normal_at_center(&self) -> DVec3 {
        let center = self.center_of_mass();
        self.normal_at(center)
    }

    pub fn workplane(&self) -> Workplane {
        const NORMAL_DIFF_TOLERANCE: f64 = 0.0001;

        let center = self.center_of_mass();
        let normal = self.normal_at(center);
        let mut x_dir = dvec3(0.0, 0.0, 1.0).cross(normal);

        if x_dir.length() < NORMAL_DIFF_TOLERANCE {
            // The normal of this face is too close to the same direction
            // as the global Z axis. Use the global X axis for X instead.
            x_dir = dvec3(1.0, 0.0, 0.0);
        }

        let mut workplane = Workplane::new(x_dir, normal);
        workplane.set_translation(center);
        workplane
    }

    pub fn union(&self, other: &Face) -> CompoundFace {
        let inner_shape = ffi::cast_face_to_shape(&self.inner);
        let other_inner_shape = ffi::cast_face_to_shape(&other.inner);

        let mut fuse_operation = ffi::BRepAlgoAPI_Fuse_ctor(inner_shape, other_inner_shape);

        let fuse_shape = fuse_operation.pin_mut().Shape();

        let compound = ffi::TopoDS_cast_to_compound(fuse_shape);

        CompoundFace::from_compound(compound)
    }

    #[must_use]
    pub fn intersect(&self, other: &Face) -> CompoundFace {
        let inner_shape = ffi::cast_face_to_shape(&self.inner);
        let other_inner_shape = ffi::cast_face_to_shape(&other.inner);

        let mut common_operation = ffi::BRepAlgoAPI_Common_ctor(inner_shape, other_inner_shape);

        let common_shape = common_operation.pin_mut().Shape();

        let compound = ffi::TopoDS_cast_to_compound(common_shape);

        CompoundFace::from_compound(compound)
    }

    pub fn subtract(&self, other: &Face) -> CompoundFace {
        let inner_shape = ffi::cast_face_to_shape(&self.inner);
        let other_inner_shape = ffi::cast_face_to_shape(&other.inner);

        let mut fuse_operation = ffi::BRepAlgoAPI_Cut_ctor(inner_shape, other_inner_shape);

        let cut_shape = fuse_operation.pin_mut().Shape();

        let compound = ffi::TopoDS_cast_to_compound(cut_shape);

        CompoundFace::from_compound(compound)
    }

    pub fn surface_area(&self) -> f64 {
        let mut props = ffi::GProp_GProps_ctor();

        let inner_shape = ffi::cast_face_to_shape(&self.inner);
        ffi::BRepGProp_SurfaceProperties(inner_shape, props.pin_mut());

        // Returns surface area, obviously.
        props.Mass()
    }

    pub fn orientation(&self) -> FaceOrientation {
        FaceOrientation::from(self.inner.Orientation())
    }

    #[must_use]
    pub fn outer_wire(&self) -> Wire {
        let inner = ffi::outer_wire(&self.inner);

        Wire { inner }
    }

    /// Get the underlying surface type of this face.
    ///
    /// This method analyzes the geometric surface that defines this topological
    /// face and returns detailed information about its type and parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use opencascade::primitives::{Face, SurfaceType};
    ///
    /// // For sheet metal applications, check if a face is a bend:
    /// fn is_bend(face: &Face) -> bool {
    ///     matches!(face.surface_type(), SurfaceType::Cylinder { .. })
    /// }
    /// ```
    pub fn surface_type(&self) -> SurfaceType {
        let surface = ffi::BRepAdaptor_Surface_ctor(&self.inner, true);
        let surface_type = surface.GetType();

        match surface_type {
            ffi::GeomAbs_SurfaceType::GeomAbs_Plane => {
                let plane = ffi::BRepAdaptor_Surface_Plane(&surface);
                let position = ffi::gp_Pln_Position(&plane);
                let location = ffi::gp_Ax3_Location(&position);
                let direction = ffi::gp_Ax3_Direction(&position);

                SurfaceType::Plane {
                    origin: dvec3(location.X(), location.Y(), location.Z()),
                    normal: dvec3(direction.X(), direction.Y(), direction.Z()),
                }
            }
            ffi::GeomAbs_SurfaceType::GeomAbs_Cylinder => {
                let cylinder = ffi::BRepAdaptor_Surface_Cylinder(&surface);
                let axis = ffi::gp_Cylinder_Axis(&cylinder);
                let location = ffi::gp_Ax1_Location(&axis);
                let direction = ffi::gp_Ax1_Direction(&axis);

                SurfaceType::Cylinder {
                    axis_origin: dvec3(location.X(), location.Y(), location.Z()),
                    axis_direction: dvec3(direction.X(), direction.Y(), direction.Z()),
                    radius: cylinder.Radius(),
                }
            }
            ffi::GeomAbs_SurfaceType::GeomAbs_Cone => {
                let cone = ffi::BRepAdaptor_Surface_Cone(&surface);
                let apex = ffi::gp_Cone_Apex(&cone);
                let axis = ffi::gp_Cone_Axis(&cone);
                let location = ffi::gp_Ax1_Location(&axis);
                let direction = ffi::gp_Ax1_Direction(&axis);

                SurfaceType::Cone {
                    apex: dvec3(apex.X(), apex.Y(), apex.Z()),
                    axis_origin: dvec3(location.X(), location.Y(), location.Z()),
                    axis_direction: dvec3(direction.X(), direction.Y(), direction.Z()),
                    semi_angle: cone.SemiAngle(),
                    ref_radius: cone.RefRadius(),
                }
            }
            ffi::GeomAbs_SurfaceType::GeomAbs_Sphere => {
                let sphere = ffi::BRepAdaptor_Surface_Sphere(&surface);
                let location = ffi::gp_Sphere_Location(&sphere);

                SurfaceType::Sphere {
                    center: dvec3(location.X(), location.Y(), location.Z()),
                    radius: sphere.Radius(),
                }
            }
            ffi::GeomAbs_SurfaceType::GeomAbs_Torus => {
                let torus = ffi::BRepAdaptor_Surface_Torus(&surface);
                let location = ffi::gp_Torus_Location(&torus);
                let axis = ffi::gp_Torus_Axis(&torus);
                let direction = ffi::gp_Ax1_Direction(&axis);

                SurfaceType::Torus {
                    center: dvec3(location.X(), location.Y(), location.Z()),
                    axis_direction: dvec3(direction.X(), direction.Y(), direction.Z()),
                    major_radius: torus.MajorRadius(),
                    minor_radius: torus.MinorRadius(),
                }
            }
            ffi::GeomAbs_SurfaceType::GeomAbs_BezierSurface => SurfaceType::Bezier,
            ffi::GeomAbs_SurfaceType::GeomAbs_BSplineSurface => SurfaceType::BSpline,
            ffi::GeomAbs_SurfaceType::GeomAbs_SurfaceOfRevolution => SurfaceType::Revolution,
            ffi::GeomAbs_SurfaceType::GeomAbs_SurfaceOfExtrusion => SurfaceType::Extrusion,
            ffi::GeomAbs_SurfaceType::GeomAbs_OffsetSurface => SurfaceType::Offset,
            ffi::GeomAbs_SurfaceType::GeomAbs_OtherSurface => SurfaceType::Other,
            ffi::GeomAbs_SurfaceType { repr } => {
                panic!("GeomAbs_SurfaceType had an unrepresentable value: {repr}")
            }
        }
    }

    /// Get the parametric bounds of this face.
    ///
    /// Returns `(u_min, u_max, v_min, v_max)` representing the parameter space
    /// of the underlying surface trimmed to this face.
    ///
    /// For cylindrical surfaces:
    /// - U is the angular parameter (in radians)
    /// - V is the axial parameter (along the cylinder axis)
    ///
    /// The angular extent of a cylindrical face is `u_max - u_min`.
    ///
    /// Note: This uses the trimmed bounds (not natural domain), so for a
    /// cylindrical face trimmed to a 90-degree arc, u_max - u_min = PI/2.
    pub fn parametric_bounds(&self) -> (f64, f64, f64, f64) {
        // Use false for restriction to get the actual trimmed bounds,
        // not the natural domain (which for a cylinder is 0 to 2*PI)
        let surface = ffi::BRepAdaptor_Surface_ctor(&self.inner, false);
        (
            surface.FirstUParameter(),
            surface.LastUParameter(),
            surface.FirstVParameter(),
            surface.LastVParameter(),
        )
    }
}

pub struct CompoundFace {
    inner: UniquePtr<ffi::TopoDS_Compound>,
}

impl AsRef<CompoundFace> for CompoundFace {
    fn as_ref(&self) -> &CompoundFace {
        self
    }
}

impl From<Face> for CompoundFace {
    fn from(face: Face) -> Self {
        let face = ffi::cast_face_to_shape(&face.inner);
        let mut compound = ffi::TopoDS_Compound_ctor();
        let brep_builder = ffi::BRep_Builder_ctor();
        let topo_builder = ffi::BRep_Builder_upcast_to_topods_builder(&brep_builder);
        topo_builder.MakeCompound(compound.pin_mut());
        let mut compound_shape = ffi::TopoDS_Compound_as_shape(compound);
        topo_builder.Add(compound_shape.pin_mut(), face);
        Self::from_compound(ffi::TopoDS_cast_to_compound(&compound_shape))
    }
}

impl CompoundFace {
    pub(crate) fn from_compound(compound: &ffi::TopoDS_Compound) -> Self {
        let inner = ffi::TopoDS_Compound_to_owned(compound);

        Self { inner }
    }

    #[must_use]
    pub fn clean(&self) -> Self {
        let shape = ffi::cast_compound_to_shape(&self.inner);
        let shape = Shape::from_shape(shape).clean();

        let compound = ffi::TopoDS_cast_to_compound(&shape.inner);

        Self::from_compound(compound)
    }

    #[must_use]
    pub fn extrude(&self, dir: DVec3) -> Shape {
        let prism_vec = make_vec(dir);

        let copy = false;
        let canonize = true;

        let inner_shape = ffi::cast_compound_to_shape(&self.inner);

        let mut make_solid =
            ffi::BRepPrimAPI_MakePrism_ctor(inner_shape, &prism_vec, copy, canonize);
        let extruded_shape = make_solid.pin_mut().Shape();

        Shape::from_shape(extruded_shape)
    }

    #[must_use]
    pub fn revolve(&self, origin: DVec3, axis: DVec3, angle: Option<Angle>) -> Shape {
        let revol_axis = make_axis_1(origin, axis);

        let angle = angle.map(Angle::radians).unwrap_or(std::f64::consts::PI * 2.0);
        let copy = false;

        let inner_shape = ffi::cast_compound_to_shape(&self.inner);

        let mut make_solid = ffi::BRepPrimAPI_MakeRevol_ctor(inner_shape, &revol_axis, angle, copy);
        let revolved_shape = make_solid.pin_mut().Shape();

        Shape::from_shape(revolved_shape)
    }

    #[must_use]
    pub fn union(&self, other: &CompoundFace) -> CompoundFace {
        let inner_shape = ffi::cast_compound_to_shape(&self.inner);
        let other_inner_shape = ffi::cast_compound_to_shape(&other.inner);

        let mut fuse_operation = ffi::BRepAlgoAPI_Fuse_ctor(inner_shape, other_inner_shape);

        let fuse_shape = fuse_operation.pin_mut().Shape();

        let compound = ffi::TopoDS_cast_to_compound(fuse_shape);

        CompoundFace::from_compound(compound)
    }

    #[must_use]
    pub fn intersect(&self, other: &CompoundFace) -> CompoundFace {
        let inner_shape = ffi::cast_compound_to_shape(&self.inner);
        let other_inner_shape = ffi::cast_compound_to_shape(&other.inner);

        let mut common_operation = ffi::BRepAlgoAPI_Common_ctor(inner_shape, other_inner_shape);

        let common_shape = common_operation.pin_mut().Shape();

        let compound = ffi::TopoDS_cast_to_compound(common_shape);

        CompoundFace::from_compound(compound)
    }

    #[must_use]
    pub fn subtract(&self, other: &CompoundFace) -> CompoundFace {
        let inner_shape = ffi::cast_compound_to_shape(&self.inner);
        let other_inner_shape = ffi::cast_compound_to_shape(&other.inner);

        let mut fuse_operation = ffi::BRepAlgoAPI_Cut_ctor(inner_shape, other_inner_shape);

        let cut_shape = fuse_operation.pin_mut().Shape();

        let compound = ffi::TopoDS_cast_to_compound(cut_shape);

        CompoundFace::from_compound(compound)
    }

    pub fn set_global_translation(&mut self, translation: DVec3) {
        let shape = ffi::cast_compound_to_shape(&self.inner);
        let mut shape = Shape::from_shape(shape);

        shape.set_global_translation(translation);

        let compound = ffi::TopoDS_cast_to_compound(&shape.inner);
        *self = Self::from_compound(compound);
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FaceOrientation {
    Forward,
    Reversed,
    Internal,
    External,
}

impl From<ffi::TopAbs_Orientation> for FaceOrientation {
    fn from(orientation: ffi::TopAbs_Orientation) -> Self {
        match orientation {
            ffi::TopAbs_Orientation::TopAbs_FORWARD => Self::Forward,
            ffi::TopAbs_Orientation::TopAbs_REVERSED => Self::Reversed,
            ffi::TopAbs_Orientation::TopAbs_INTERNAL => Self::Internal,
            ffi::TopAbs_Orientation::TopAbs_EXTERNAL => Self::External,
            ffi::TopAbs_Orientation { repr } => {
                panic!("TopAbs_Orientation had an unrepresentable value: {repr}")
            },
        }
    }
}

/// The underlying geometric surface type of a face.
///
/// This enum provides access to the parametric surface definition that
/// underlies a topological face. For sheet metal applications, the most
/// relevant types are:
/// - [`SurfaceType::Plane`] - flat faces
/// - [`SurfaceType::Cylinder`] - bend surfaces
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceType {
    /// A planar surface defined by an origin point and a normal direction.
    Plane {
        /// A point on the plane (the plane's origin in its local coordinate system).
        origin: DVec3,
        /// The normal vector perpendicular to the plane.
        normal: DVec3,
    },

    /// A cylindrical surface defined by an axis and radius.
    /// Common for bends in sheet metal parts.
    Cylinder {
        /// Origin point of the cylinder axis.
        axis_origin: DVec3,
        /// Direction of the cylinder axis.
        axis_direction: DVec3,
        /// Radius of the cylinder.
        radius: f64,
    },

    /// A conical surface defined by an apex, axis, and half-angle.
    Cone {
        /// The apex (tip) of the cone.
        apex: DVec3,
        /// Origin point of the cone axis.
        axis_origin: DVec3,
        /// Direction of the cone axis.
        axis_direction: DVec3,
        /// The semi-angle of the cone in radians.
        semi_angle: f64,
        /// Reference radius at the origin location.
        ref_radius: f64,
    },

    /// A spherical surface defined by a center and radius.
    Sphere {
        /// Center point of the sphere.
        center: DVec3,
        /// Radius of the sphere.
        radius: f64,
    },

    /// A toroidal (donut-shaped) surface defined by axis and radii.
    Torus {
        /// Center point of the torus.
        center: DVec3,
        /// Direction of the torus axis.
        axis_direction: DVec3,
        /// Major radius (distance from center to tube center).
        major_radius: f64,
        /// Minor radius (radius of the tube).
        minor_radius: f64,
    },

    /// A Bezier surface (polynomial).
    Bezier,

    /// A B-spline surface (NURBS).
    BSpline,

    /// A surface of revolution (generated by rotating a curve).
    Revolution,

    /// A surface of extrusion (generated by extruding a curve).
    Extrusion,

    /// An offset surface (parallel to another surface).
    Offset,

    /// Any other surface type not specifically categorized.
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let face = Workplane::xy().rect(7.0, 5.0).to_face();
        assert!(
            (face.surface_area() - 35.0).abs() <= 0.00001,
            "Expected surface_area() to be ~35.0, was actually {}",
            face.surface_area()
        );
    }

    #[test]
    fn test_surface_type_plane() {
        // Create a rectangular planar face on the XY plane
        let face = Workplane::xy().rect(10.0, 5.0).to_face();

        match face.surface_type() {
            SurfaceType::Plane { origin: _, normal } => {
                // Normal should be pointing in Z direction (approximately)
                assert!(
                    (normal.z.abs() - 1.0).abs() < 0.001,
                    "Expected normal to be along Z axis, got {:?}",
                    normal
                );
            }
            other => panic!("Expected Plane surface type, got {:?}", other),
        }
    }

    #[test]
    fn test_surface_type_cylinder() {
        // Create a cylinder primitive (radius 5, height 10)
        let cylinder = Shape::cylinder_radius_height(5.0, 10.0);

        // Find a cylindrical face in the resulting shape
        let mut found_cylinder = false;
        for face in cylinder.faces() {
            if let SurfaceType::Cylinder {
                axis_direction,
                radius,
                ..
            } = face.surface_type()
            {
                // Axis should be along Z
                assert!(
                    (axis_direction.z.abs() - 1.0).abs() < 0.001,
                    "Expected cylinder axis along Z, got {:?}",
                    axis_direction
                );
                // Radius should be 5.0
                assert!(
                    (radius - 5.0).abs() < 0.001,
                    "Expected radius 5.0, got {}",
                    radius
                );
                found_cylinder = true;
                break;
            }
        }
        assert!(found_cylinder, "Expected to find a cylindrical face");
    }
}
