use crate::primitives::{make_axis_2, make_point};
use cxx::UniquePtr;
use glam::{dvec3, DVec3};
use opencascade_sys::ffi;

use super::make_vec;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EdgeType {
    Line,
    Circle,
    Ellipse,
    Hyperbola,
    Parabola,
    BezierCurve,
    BSplineCurve,
    OffsetCurve,
    OtherCurve,
}

impl From<ffi::GeomAbs_CurveType> for EdgeType {
    fn from(curve_type: ffi::GeomAbs_CurveType) -> Self {
        match curve_type {
            ffi::GeomAbs_CurveType::GeomAbs_Line => Self::Line,
            ffi::GeomAbs_CurveType::GeomAbs_Circle => Self::Circle,
            ffi::GeomAbs_CurveType::GeomAbs_Ellipse => Self::Ellipse,
            ffi::GeomAbs_CurveType::GeomAbs_Hyperbola => Self::Hyperbola,
            ffi::GeomAbs_CurveType::GeomAbs_Parabola => Self::Parabola,
            ffi::GeomAbs_CurveType::GeomAbs_BezierCurve => Self::BezierCurve,
            ffi::GeomAbs_CurveType::GeomAbs_BSplineCurve => Self::BSplineCurve,
            ffi::GeomAbs_CurveType::GeomAbs_OffsetCurve => Self::OffsetCurve,
            ffi::GeomAbs_CurveType::GeomAbs_OtherCurve => Self::OtherCurve,
            ffi::GeomAbs_CurveType { repr } => panic!("Unexpected curve type: {repr}"),
        }
    }
}

pub struct Edge {
    pub(crate) inner: UniquePtr<ffi::TopoDS_Edge>,
}

impl AsRef<Edge> for Edge {
    fn as_ref(&self) -> &Edge {
        self
    }
}

impl Edge {
    pub(crate) fn from_edge(edge: &ffi::TopoDS_Edge) -> Self {
        let inner = ffi::TopoDS_Edge_to_owned(edge);

        Self { inner }
    }

    /// Get a reference to the underlying OpenCascade edge.
    pub fn inner(&self) -> &ffi::TopoDS_Edge {
        &self.inner
    }

    /// Check if two edges refer to the same underlying TopoDS edge.
    pub fn is_same(&self, other: &Edge) -> bool {
        ffi::TopoDS_Edge_IsSame(self.inner(), other.inner())
    }

    fn from_make_edge(mut make_edge: UniquePtr<ffi::BRepBuilderAPI_MakeEdge>) -> Self {
        Self::from_edge(make_edge.pin_mut().Edge())
    }

    pub fn segment(p1: DVec3, p2: DVec3) -> Self {
        let make_edge =
            ffi::BRepBuilderAPI_MakeEdge_gp_Pnt_gp_Pnt(&make_point(p1), &make_point(p2));

        Self::from_make_edge(make_edge)
    }

    pub fn bezier(points: impl IntoIterator<Item = DVec3>) -> Self {
        let points: Vec<_> = points.into_iter().collect();
        let mut array = ffi::TColgp_HArray1OfPnt_ctor(1, points.len() as i32);
        for (index, point) in points.into_iter().enumerate() {
            array.pin_mut().SetValue(index as i32 + 1, &make_point(point));
        }

        let bezier = ffi::Geom_BezierCurve_ctor_points(&array);
        let bezier_handle = ffi::Geom_BezierCurve_to_handle(bezier);
        let curve_handle = ffi::new_HandleGeomCurve_from_HandleGeom_BezierCurve(&bezier_handle);

        let mut make_edge = ffi::BRepBuilderAPI_MakeEdge_HandleGeomCurve(&curve_handle);
        let edge = make_edge.pin_mut().Edge();
        Self::from_edge(edge)
    }

    pub fn circle(center: DVec3, normal: DVec3, radius: f64) -> Self {
        let axis = make_axis_2(center, normal);

        let make_circle = ffi::gp_Circ_ctor(&axis, radius);
        let make_edge = ffi::BRepBuilderAPI_MakeEdge_circle(&make_circle);

        Self::from_make_edge(make_edge)
    }

    pub fn ellipse() {}

    pub fn spline_from_points(
        points: impl IntoIterator<Item = DVec3>,
        tangents: Option<(DVec3, DVec3)>,
    ) -> Self {
        let points: Vec<_> = points.into_iter().collect();
        let mut array = ffi::TColgp_HArray1OfPnt_ctor(1, points.len() as i32);
        for (index, point) in points.into_iter().enumerate() {
            array.pin_mut().SetValue(index as i32 + 1, &make_point(point));
        }
        let array_handle = ffi::new_HandleTColgpHArray1OfPnt_from_TColgpHArray1OfPnt(array);

        let periodic = false;
        let tolerance = 1.0e-7;
        let mut interpolate = ffi::GeomAPI_Interpolate_ctor(&array_handle, periodic, tolerance);
        if let Some((t_start, t_end)) = tangents {
            interpolate.pin_mut().Load(&make_vec(t_start), &make_vec(t_end), true);
        }

        interpolate.pin_mut().Perform();
        let bspline_handle = ffi::GeomAPI_Interpolate_Curve(&interpolate);
        let curve_handle = ffi::new_HandleGeomCurve_from_HandleGeom_BSplineCurve(&bspline_handle);

        let mut make_edge = ffi::BRepBuilderAPI_MakeEdge_HandleGeomCurve(&curve_handle);
        let edge = make_edge.pin_mut().Edge();
        Self::from_edge(edge)
    }

    /// Create a B-spline edge from control points (poles), knots, multiplicities, and degree.
    ///
    /// This is useful for importing B-splines from formats like DXF that provide these parameters
    /// directly rather than points to interpolate.
    ///
    /// # Arguments
    /// * `poles` - Control points of the B-spline
    /// * `knots` - Unique knot values (strictly increasing)
    /// * `multiplicities` - Multiplicity of each knot (how many times it repeats)
    /// * `degree` - Polynomial degree of the B-spline
    /// * `periodic` - Whether the curve is periodic (closed)
    pub fn bspline(
        poles: impl IntoIterator<Item = DVec3>,
        knots: impl IntoIterator<Item = f64>,
        multiplicities: impl IntoIterator<Item = i32>,
        degree: i32,
        periodic: bool,
    ) -> Self {
        let poles: Vec<_> = poles.into_iter().collect();
        let knots: Vec<_> = knots.into_iter().collect();
        let multiplicities: Vec<_> = multiplicities.into_iter().collect();

        // Create poles array
        let mut poles_array = ffi::TColgp_Array1OfPnt_ctor(1, poles.len() as i32);
        for (i, point) in poles.into_iter().enumerate() {
            ffi::TColgp_Array1OfPnt_SetValue(poles_array.pin_mut(), i as i32 + 1, &make_point(point));
        }

        // Create knots array
        let mut knots_array = ffi::TColStd_Array1OfReal_ctor(1, knots.len() as i32);
        for (i, knot) in knots.into_iter().enumerate() {
            ffi::TColStd_Array1OfReal_SetValue(knots_array.pin_mut(), i as i32 + 1, knot);
        }

        // Create multiplicities array
        let mut mults_array = ffi::TColStd_Array1OfInteger_ctor(1, multiplicities.len() as i32);
        for (i, mult) in multiplicities.into_iter().enumerate() {
            ffi::TColStd_Array1OfInteger_SetValue(mults_array.pin_mut(), i as i32 + 1, mult);
        }

        // Create the B-spline curve
        let bspline = ffi::Geom_BSplineCurve_ctor(
            &poles_array,
            &knots_array,
            &mults_array,
            degree,
            periodic,
        );
        let bspline_handle = ffi::Geom_BSplineCurve_to_handle(bspline);
        let curve_handle = ffi::new_HandleGeomCurve_from_HandleGeom_BSplineCurve(&bspline_handle);

        let mut make_edge = ffi::BRepBuilderAPI_MakeEdge_HandleGeomCurve(&curve_handle);
        let edge = make_edge.pin_mut().Edge();
        Self::from_edge(edge)
    }

    /// Create a rational B-spline (NURBS) edge with weights.
    ///
    /// This is useful for exact representations of conic sections (circles, ellipses, etc.)
    /// and for importing NURBS curves from CAD formats.
    ///
    /// # Arguments
    /// * `poles` - Control points of the B-spline
    /// * `weights` - Weight for each control point (all equal = non-rational B-spline)
    /// * `knots` - Unique knot values (strictly increasing)
    /// * `multiplicities` - Multiplicity of each knot
    /// * `degree` - Polynomial degree of the B-spline
    /// * `periodic` - Whether the curve is periodic (closed)
    pub fn nurbs(
        poles: impl IntoIterator<Item = DVec3>,
        weights: impl IntoIterator<Item = f64>,
        knots: impl IntoIterator<Item = f64>,
        multiplicities: impl IntoIterator<Item = i32>,
        degree: i32,
        periodic: bool,
    ) -> Self {
        let poles: Vec<_> = poles.into_iter().collect();
        let weights: Vec<_> = weights.into_iter().collect();
        let knots: Vec<_> = knots.into_iter().collect();
        let multiplicities: Vec<_> = multiplicities.into_iter().collect();

        // Create poles array
        let mut poles_array = ffi::TColgp_Array1OfPnt_ctor(1, poles.len() as i32);
        for (i, point) in poles.into_iter().enumerate() {
            ffi::TColgp_Array1OfPnt_SetValue(poles_array.pin_mut(), i as i32 + 1, &make_point(point));
        }

        // Create weights array
        let mut weights_array = ffi::TColStd_Array1OfReal_ctor(1, weights.len() as i32);
        for (i, weight) in weights.into_iter().enumerate() {
            ffi::TColStd_Array1OfReal_SetValue(weights_array.pin_mut(), i as i32 + 1, weight);
        }

        // Create knots array
        let mut knots_array = ffi::TColStd_Array1OfReal_ctor(1, knots.len() as i32);
        for (i, knot) in knots.into_iter().enumerate() {
            ffi::TColStd_Array1OfReal_SetValue(knots_array.pin_mut(), i as i32 + 1, knot);
        }

        // Create multiplicities array
        let mut mults_array = ffi::TColStd_Array1OfInteger_ctor(1, multiplicities.len() as i32);
        for (i, mult) in multiplicities.into_iter().enumerate() {
            ffi::TColStd_Array1OfInteger_SetValue(mults_array.pin_mut(), i as i32 + 1, mult);
        }

        // Create the NURBS curve
        let bspline = ffi::Geom_BSplineCurve_ctor_weighted(
            &poles_array,
            &weights_array,
            &knots_array,
            &mults_array,
            degree,
            periodic,
        );
        let bspline_handle = ffi::Geom_BSplineCurve_to_handle(bspline);
        let curve_handle = ffi::new_HandleGeomCurve_from_HandleGeom_BSplineCurve(&bspline_handle);

        let mut make_edge = ffi::BRepBuilderAPI_MakeEdge_HandleGeomCurve(&curve_handle);
        let edge = make_edge.pin_mut().Edge();
        Self::from_edge(edge)
    }

    pub fn arc(p1: DVec3, p2: DVec3, p3: DVec3) -> Self {
        let make_arc = ffi::GC_MakeArcOfCircle_point_point_point(
            &make_point(p1),
            &make_point(p2),
            &make_point(p3),
        );

        let make_edge = ffi::BRepBuilderAPI_MakeEdge_HandleGeomCurve(
            &ffi::new_HandleGeomCurve_from_HandleGeom_TrimmedCurve(&ffi::GC_MakeArcOfCircle_Value(
                &make_arc,
            )),
        );

        Self::from_make_edge(make_edge)
    }

    pub fn start_point(&self) -> DVec3 {
        let curve = ffi::BRepAdaptor_Curve_ctor(&self.inner);
        let start_param = curve.FirstParameter();
        let point = ffi::BRepAdaptor_Curve_value(&curve, start_param);

        dvec3(point.X(), point.Y(), point.Z())
    }

    pub fn end_point(&self) -> DVec3 {
        let curve = ffi::BRepAdaptor_Curve_ctor(&self.inner);
        let last_param = curve.LastParameter();
        let point = ffi::BRepAdaptor_Curve_value(&curve, last_param);

        dvec3(point.X(), point.Y(), point.Z())
    }

    pub fn approximation_segments(&self) -> ApproximationSegmentIterator {
        let adaptor_curve = ffi::BRepAdaptor_Curve_ctor(&self.inner);
        let approximator = ffi::GCPnts_TangentialDeflection_ctor(&adaptor_curve, 0.1, 0.1);

        ApproximationSegmentIterator { count: 1, approximator }
    }

    pub fn tangent_arc(_p1: DVec3, _tangent: DVec3, _p3: DVec3) {}

    pub fn edge_type(&self) -> EdgeType {
        let curve = ffi::BRepAdaptor_Curve_ctor(&self.inner);

        EdgeType::from(curve.GetType())
    }
}

pub struct ApproximationSegmentIterator {
    count: usize,
    approximator: UniquePtr<ffi::GCPnts_TangentialDeflection>,
}

impl Iterator for ApproximationSegmentIterator {
    type Item = DVec3;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count <= self.approximator.NbPoints() as usize {
            let point =
                ffi::GCPnts_TangentialDeflection_Value(&self.approximator, self.count as i32);

            self.count += 1;
            Some(dvec3(point.X(), point.Y(), point.Z()))
        } else {
            None
        }
    }
}
