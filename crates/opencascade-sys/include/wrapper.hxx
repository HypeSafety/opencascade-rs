#include "rust/cxx.h"
#include <BOPAlgo_GlueEnum.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Curve2d.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepAlgoAPI_Section.hxx>
#include <BRepBndLib.hxx>
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeShapeOnMesh.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepFeat_MakeCylindricalHole.hxx>
#include <BRepFeat_MakeDPrism.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepFilletAPI_MakeFillet2d.hxx>
#include <BRepGProp.hxx>
#include <BRepGProp_Face.hxx>
#include <BRepIntCurveSurface_Inter.hxx>
#include <BRepLib.hxx>
#include <BRepLProp_SLProps.hxx>
#include <BRepLib_ToolTriangulatedShape.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepOffsetAPI_MakeOffset.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepOffset_Mode.hxx>
#include <BRepOffsetAPI_MakePipe.hxx>
#include <BRepOffsetAPI_MakePipeShell.hxx>
#include <BRepOffsetAPI_MakeThickSolid.hxx>
#include <BRepOffsetAPI_ThruSections.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRepTools.hxx>
#include <GCE2d_MakeSegment.hxx>
#include <GCPnts_AbscissaPoint.hxx>
#include <GCPnts_TangentialDeflection.hxx>
#include <GC_MakeArcOfCircle.hxx>
#include <GC_MakeSegment.hxx>
#include <GProp_GProps.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom2dConvert.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Lin2d.hxx>
#include <GeomAPI_Interpolate.hxx>
#include <GeomAPI_ProjectPointOnSurf.hxx>
#include <GeomAbs_CurveType.hxx>
#include <GeomAbs_JoinType.hxx>
#include <GeomAbs_SurfaceType.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_Line.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_Surface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <IGESControl_Reader.hxx>
#include <IGESControl_Writer.hxx>
#include <Law_Function.hxx>
#include <Law_Interpol.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <Poly_Connect.hxx>
#include <STEPControl_Reader.hxx>
#include <STEPControl_Writer.hxx>
#include <ShapeAnalysis_FreeBounds.hxx>
#include <ShapeFix_Shape.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <Standard_Failure.hxx>
#include <Standard_Type.hxx>
#include <StlAPI_Writer.hxx>
#include <TColgp_Array1OfDir.hxx>
#include <TColgp_HArray1OfPnt.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_HSequenceOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <gp.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Circ.hxx>
#include <gp_Cone.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Lin.hxx>
#include <gp_Pln.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>

// Generic template constructor
template <typename T, typename... Args> std::unique_ptr<T> construct_unique(Args... args) {
  return std::unique_ptr<T>(new T(args...));
}

// Generic List
template <typename T> std::unique_ptr<std::vector<T>> list_to_vector(const NCollection_List<T> &list) {
  return std::unique_ptr<std::vector<T>>(new std::vector<T>(list.begin(), list.end()));
}

// Handles
typedef opencascade::handle<Standard_Type> HandleStandardType;
typedef opencascade::handle<Geom_Curve> HandleGeomCurve;
typedef opencascade::handle<Geom_BSplineCurve> HandleGeomBSplineCurve;
typedef opencascade::handle<Geom_BezierCurve> HandleGeomBezierCurve;
typedef opencascade::handle<Geom_TrimmedCurve> HandleGeomTrimmedCurve;
typedef opencascade::handle<Geom_Line> HandleGeomLine;
typedef opencascade::handle<Geom_Surface> HandleGeomSurface;
typedef opencascade::handle<Geom_BezierSurface> HandleGeomBezierSurface;
typedef opencascade::handle<Geom_Plane> HandleGeomPlane;
typedef opencascade::handle<Geom2d_Curve> HandleGeom2d_Curve;
typedef opencascade::handle<Geom2d_Line> HandleGeom2d_Line;
typedef opencascade::handle<Geom2d_Circle> HandleGeom2d_Circle;
typedef opencascade::handle<Geom2d_BSplineCurve> HandleGeom2d_BSplineCurve;
typedef opencascade::handle<Geom2d_Ellipse> HandleGeom2d_Ellipse;
typedef opencascade::handle<Geom2d_TrimmedCurve> HandleGeom2d_TrimmedCurve;
typedef opencascade::handle<Geom_CylindricalSurface> HandleGeom_CylindricalSurface;
typedef opencascade::handle<Poly_Triangulation> HandlePoly_Triangulation;
typedef opencascade::handle<TopTools_HSequenceOfShape> HandleTopTools_HSequenceOfShape;
typedef opencascade::handle<Law_Function> HandleLawFunction;

typedef opencascade::handle<TColgp_HArray1OfPnt> Handle_TColgpHArray1OfPnt;

inline std::unique_ptr<Handle_TColgpHArray1OfPnt>
new_HandleTColgpHArray1OfPnt_from_TColgpHArray1OfPnt(std::unique_ptr<TColgp_HArray1OfPnt> array) {
  return std::unique_ptr<Handle_TColgpHArray1OfPnt>(new Handle_TColgpHArray1OfPnt(array.release()));
}

// Handle stuff
template <typename T> const T &handle_try_deref(const opencascade::handle<T> &handle) {
  if (handle.IsNull()) {
    throw std::runtime_error("null handle dereference");
  }
  return *handle;
}

// Thread-local storage for OCCT exception messages
// This allows Rust to retrieve error details after a function returns nullptr
inline std::string& get_occt_last_error_ref() {
  static thread_local std::string occt_last_error;
  return occt_last_error;
}

inline rust::String get_occt_last_error() {
  return rust::String(get_occt_last_error_ref());
}

inline void clear_occt_last_error() {
  get_occt_last_error_ref().clear();
}

inline const HandleStandardType &DynamicType(const HandleGeomSurface &surface) { return surface->DynamicType(); }

inline rust::String type_name(const HandleStandardType &handle) { return std::string(handle->Name()); }

inline std::unique_ptr<gp_Pnt> HandleGeomCurve_Value(const HandleGeomCurve &curve, const Standard_Real U) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(curve->Value(U)));
}

inline std::unique_ptr<gp_Pnt> GCPnts_TangentialDeflection_Value(const GCPnts_TangentialDeflection &approximator,
                                                                 Standard_Integer i) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(approximator.Value(i)));
}

inline std::unique_ptr<HandleGeomPlane> new_HandleGeomPlane_from_HandleGeomSurface(const HandleGeomSurface &surface) {
  HandleGeomPlane plane_handle = opencascade::handle<Geom_Plane>::DownCast(surface);
  return std::unique_ptr<HandleGeomPlane>(new opencascade::handle<Geom_Plane>(plane_handle));
}

// Collections
inline void shape_list_append_face(TopTools_ListOfShape &list, const TopoDS_Face &face) { list.Append(face); }

// Geometry
inline const gp_Pnt &handle_geom_plane_location(const HandleGeomPlane &plane) { return plane->Location(); }

inline std::unique_ptr<HandleGeom_CylindricalSurface> Geom_CylindricalSurface_ctor(const gp_Ax3 &axis, double radius) {
  return std::unique_ptr<HandleGeom_CylindricalSurface>(
      new opencascade::handle<Geom_CylindricalSurface>(new Geom_CylindricalSurface(axis, radius)));
}

inline std::unique_ptr<HandleGeomBSplineCurve> GeomAPI_Interpolate_Curve(const GeomAPI_Interpolate &interpolate) {
  return std::unique_ptr<HandleGeomBSplineCurve>(new opencascade::handle<Geom_BSplineCurve>(interpolate.Curve()));
}

inline std::unique_ptr<HandleGeomBezierCurve>
Geom_BezierCurve_to_handle(std::unique_ptr<Geom_BezierCurve> bezier_curve) {
  return std::unique_ptr<HandleGeomBezierCurve>(new HandleGeomBezierCurve(bezier_curve.release()));
}

inline std::unique_ptr<HandleGeomSurface> cylinder_to_surface(const HandleGeom_CylindricalSurface &cylinder_handle) {
  return std::unique_ptr<HandleGeomSurface>(new opencascade::handle<Geom_Surface>(cylinder_handle));
}

inline std::unique_ptr<HandleGeomBezierSurface> Geom_BezierSurface_ctor(const TColgp_Array2OfPnt &poles) {
  return std::unique_ptr<HandleGeomBezierSurface>(
      new opencascade::handle<Geom_BezierSurface>(new Geom_BezierSurface(poles)));
}

inline std::unique_ptr<HandleGeomSurface> bezier_to_surface(const HandleGeomBezierSurface &bezier_handle) {
  return std::unique_ptr<HandleGeomSurface>(new opencascade::handle<Geom_Surface>(bezier_handle));
}

inline std::unique_ptr<HandleGeom2d_Ellipse> Geom2d_Ellipse_ctor(const gp_Ax2d &axis, double major_radius,
                                                                 double minor_radius) {
  return std::unique_ptr<HandleGeom2d_Ellipse>(
      new opencascade::handle<Geom2d_Ellipse>(new Geom2d_Ellipse(axis, major_radius, minor_radius)));
}

inline std::unique_ptr<HandleGeom2d_Curve> ellipse_to_HandleGeom2d_Curve(const HandleGeom2d_Ellipse &ellipse_handle) {
  return std::unique_ptr<HandleGeom2d_Curve>(new opencascade::handle<Geom2d_Curve>(ellipse_handle));
}

inline std::unique_ptr<HandleGeom2d_TrimmedCurve> Geom2d_TrimmedCurve_ctor(const HandleGeom2d_Curve &curve, double u1,
                                                                           double u2) {
  return std::unique_ptr<HandleGeom2d_TrimmedCurve>(
      new opencascade::handle<Geom2d_TrimmedCurve>(new Geom2d_TrimmedCurve(curve, u1, u2)));
}

inline std::unique_ptr<HandleGeom2d_Curve>
HandleGeom2d_TrimmedCurve_to_curve(const HandleGeom2d_TrimmedCurve &trimmed_curve) {
  return std::unique_ptr<HandleGeom2d_Curve>(new opencascade::handle<Geom2d_Curve>(trimmed_curve));
}

inline std::unique_ptr<gp_Pnt2d> ellipse_value(const HandleGeom2d_Ellipse &ellipse, double u) {
  return std::unique_ptr<gp_Pnt2d>(new gp_Pnt2d(ellipse->Value(u)));
}

// Segment Stuff
inline std::unique_ptr<HandleGeomTrimmedCurve> GC_MakeSegment_Value(const GC_MakeSegment &segment) {
  return std::unique_ptr<HandleGeomTrimmedCurve>(new opencascade::handle<Geom_TrimmedCurve>(segment.Value()));
}

inline std::unique_ptr<HandleGeom2d_TrimmedCurve> GCE2d_MakeSegment_point_point(const gp_Pnt2d &p1,
                                                                                const gp_Pnt2d &p2) {
  return std::unique_ptr<HandleGeom2d_TrimmedCurve>(
      new opencascade::handle<Geom2d_TrimmedCurve>(GCE2d_MakeSegment(p1, p2)));
}

// Arc stuff
inline std::unique_ptr<HandleGeomTrimmedCurve> GC_MakeArcOfCircle_Value(const GC_MakeArcOfCircle &arc) {
  return std::unique_ptr<HandleGeomTrimmedCurve>(new opencascade::handle<Geom_TrimmedCurve>(arc.Value()));
}

inline std::unique_ptr<gp_Pnt> BRepAdaptor_Curve_value(const BRepAdaptor_Curve &curve, const Standard_Real U) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(curve.Value(U)));
}

// BRepLib
inline bool BRepLibBuildCurves3d(const TopoDS_Shape &shape) { return BRepLib::BuildCurves3d(shape); }

inline void MakeThickSolidByJoin(BRepOffsetAPI_MakeThickSolid &make_thick_solid, const TopoDS_Shape &shape,
                                 const TopTools_ListOfShape &closing_faces, const Standard_Real offset,
                                 const Standard_Real tolerance) {
  make_thick_solid.MakeThickSolidByJoin(shape, closing_faces, offset, tolerance);
}

// Geometric processing
inline const gp_Ax1 &gp_OX() { return gp::OX(); }
inline const gp_Ax1 &gp_OY() { return gp::OY(); }
inline const gp_Ax1 &gp_OZ() { return gp::OZ(); }

inline const gp_Dir &gp_DZ() { return gp::DZ(); }

inline std::unique_ptr<gp_Ax1> gp_Ax1_ctor(const gp_Pnt &origin, const gp_Dir &main_dir) {
  return std::unique_ptr<gp_Ax1>(new gp_Ax1(origin, main_dir));
}

inline std::unique_ptr<gp_Pnt> gp_Ax1_Location(const gp_Ax1 &axis) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(axis.Location()));
}

inline std::unique_ptr<gp_Dir> gp_Ax1_Direction(const gp_Ax1 &axis) {
  return std::unique_ptr<gp_Dir>(new gp_Dir(axis.Direction()));
}

inline std::unique_ptr<gp_Ax2> gp_Ax2_ctor(const gp_Pnt &origin, const gp_Dir &main_dir) {
  return std::unique_ptr<gp_Ax2>(new gp_Ax2(origin, main_dir));
}

inline std::unique_ptr<gp_Ax3> gp_Ax3_from_gp_Ax2(const gp_Ax2 &axis) {
  return std::unique_ptr<gp_Ax3>(new gp_Ax3(axis));
}

inline std::unique_ptr<gp_Dir> gp_Dir_ctor(double x, double y, double z) {
  return std::unique_ptr<gp_Dir>(new gp_Dir(x, y, z));
}

inline std::unique_ptr<gp_Dir2d> gp_Dir2d_ctor(double x, double y) {
  return std::unique_ptr<gp_Dir2d>(new gp_Dir2d(x, y));
}

inline std::unique_ptr<gp_Ax2d> gp_Ax2d_ctor(const gp_Pnt2d &point, const gp_Dir2d &dir) {
  return std::unique_ptr<gp_Ax2d>(new gp_Ax2d(point, dir));
}

// Law_Function stuff
inline std::unique_ptr<HandleLawFunction> Law_Function_to_handle(std::unique_ptr<Law_Function> law_function) {
  return std::unique_ptr<HandleLawFunction>(new HandleLawFunction(law_function.release()));
}

// Law_Interpol stuff
inline std::unique_ptr<Law_Function> Law_Interpol_into_Law_Function(std::unique_ptr<Law_Interpol> law_interpol) {
  return std::unique_ptr<Law_Function>(law_interpol.release());
}

// Shape stuff
inline const TopoDS_Vertex &TopoDS_cast_to_vertex(const TopoDS_Shape &shape) { return TopoDS::Vertex(shape); }
inline const TopoDS_Edge &TopoDS_cast_to_edge(const TopoDS_Shape &shape) { return TopoDS::Edge(shape); }
inline const TopoDS_Wire &TopoDS_cast_to_wire(const TopoDS_Shape &shape) { return TopoDS::Wire(shape); }
inline const TopoDS_Face &TopoDS_cast_to_face(const TopoDS_Shape &shape) { return TopoDS::Face(shape); }
inline const TopoDS_Shell &TopoDS_cast_to_shell(const TopoDS_Shape &shape) { return TopoDS::Shell(shape); }
inline const TopoDS_Solid &TopoDS_cast_to_solid(const TopoDS_Shape &shape) { return TopoDS::Solid(shape); }
inline const TopoDS_Compound &TopoDS_cast_to_compound(const TopoDS_Shape &shape) { return TopoDS::Compound(shape); }

inline const TopoDS_Shape &cast_vertex_to_shape(const TopoDS_Vertex &vertex) { return vertex; }
inline const TopoDS_Shape &cast_edge_to_shape(const TopoDS_Edge &edge) { return edge; }
inline const TopoDS_Shape &cast_wire_to_shape(const TopoDS_Wire &wire) { return wire; }
inline const TopoDS_Shape &cast_face_to_shape(const TopoDS_Face &face) { return face; }
inline const TopoDS_Shape &cast_shell_to_shape(const TopoDS_Shell &shell) { return shell; }
inline const TopoDS_Shape &cast_solid_to_shape(const TopoDS_Solid &solid) { return solid; }
inline const TopoDS_Shape &cast_compound_to_shape(const TopoDS_Compound &compound) { return compound; }

// TopoDS IsSame comparisons
inline bool TopoDS_Face_IsSame(const TopoDS_Face &f1, const TopoDS_Face &f2) { return f1.IsSame(f2); }
inline bool TopoDS_Edge_IsSame(const TopoDS_Edge &e1, const TopoDS_Edge &e2) { return e1.IsSame(e2); }

// TopoDS hash codes - uses std::hash specialization from TopoDS_*.hxx headers
inline size_t TopoDS_Edge_hash_code(const TopoDS_Edge &edge) { return std::hash<TopoDS_Edge>{}(edge); }
inline size_t TopoDS_Face_hash_code(const TopoDS_Face &face) { return std::hash<TopoDS_Face>{}(face); }

// Compound shapes
inline std::unique_ptr<TopoDS_Shape> TopoDS_Compound_as_shape(std::unique_ptr<TopoDS_Compound> compound) {
  return compound;
}

inline std::unique_ptr<TopoDS_Shape> TopoDS_Shell_as_shape(std::unique_ptr<TopoDS_Shell> shell) { return shell; }

inline const TopoDS_Builder &BRep_Builder_upcast_to_topods_builder(const BRep_Builder &builder) { return builder; }

// Transforms
inline std::unique_ptr<HandleGeomSurface> BRep_Tool_Surface(const TopoDS_Face &face) {
  return std::unique_ptr<HandleGeomSurface>(new opencascade::handle<Geom_Surface>(BRep_Tool::Surface(face)));
}

inline std::unique_ptr<HandleGeomCurve> BRep_Tool_Curve(const TopoDS_Edge &edge, Standard_Real &first,
                                                        Standard_Real &last) {
  return std::unique_ptr<HandleGeomCurve>(new opencascade::handle<Geom_Curve>(BRep_Tool::Curve(edge, first, last)));
}

// Get 2D curve of edge on face (parametric curve in surface UV space)
inline std::unique_ptr<HandleGeom2d_Curve> BRep_Tool_CurveOnSurface(
    const TopoDS_Edge &edge, const TopoDS_Face &face,
    Standard_Real &first, Standard_Real &last
) {
  return std::unique_ptr<HandleGeom2d_Curve>(
    new opencascade::handle<Geom2d_Curve>(BRep_Tool::CurveOnSurface(edge, face, first, last))
  );
}

// Evaluate 2D curve at parameter u
inline std::unique_ptr<gp_Pnt2d> HandleGeom2d_Curve_Value(const HandleGeom2d_Curve &curve, double u) {
  return std::unique_ptr<gp_Pnt2d>(new gp_Pnt2d(curve->Value(u)));
}

inline std::unique_ptr<gp_Pnt> BRep_Tool_Pnt(const TopoDS_Vertex &vertex) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(BRep_Tool::Pnt(vertex)));
}

inline std::unique_ptr<gp_Trsf> TopLoc_Location_Transformation(const TopLoc_Location &location) {
  return std::unique_ptr<gp_Trsf>(new gp_Trsf(location.Transformation()));
}

inline std::unique_ptr<HandlePoly_Triangulation>
HandlePoly_Triangulation_ctor(std::unique_ptr<Poly_Triangulation> triangulation) {
  return std::unique_ptr<HandlePoly_Triangulation>(new HandlePoly_Triangulation(triangulation.release()));
}

inline std::unique_ptr<HandlePoly_Triangulation> BRep_Tool_Triangulation(const TopoDS_Face &face,
                                                                         TopLoc_Location &location) {
  return std::unique_ptr<HandlePoly_Triangulation>(
      new opencascade::handle<Poly_Triangulation>(BRep_Tool::Triangulation(face, location)));
}

inline std::unique_ptr<TopoDS_Shape> ExplorerCurrentShape(const TopExp_Explorer &explorer) {
  return std::unique_ptr<TopoDS_Shape>(new TopoDS_Shape(explorer.Current()));
}

inline std::unique_ptr<TopoDS_Vertex> TopExp_FirstVertex(const TopoDS_Edge &edge) {
  return std::unique_ptr<TopoDS_Vertex>(new TopoDS_Vertex(TopExp::FirstVertex(edge)));
}

inline std::unique_ptr<TopoDS_Vertex> TopExp_LastVertex(const TopoDS_Edge &edge) {
  return std::unique_ptr<TopoDS_Vertex>(new TopoDS_Vertex(TopExp::LastVertex(edge)));
}

inline void TopExp_EdgeVertices(const TopoDS_Edge &edge, TopoDS_Vertex &vertex1, TopoDS_Vertex &vertex2) {
  return TopExp::Vertices(edge, vertex1, vertex2);
}

inline void TopExp_WireVertices(const TopoDS_Wire &wire, TopoDS_Vertex &vertex1, TopoDS_Vertex &vertex2) {
  return TopExp::Vertices(wire, vertex1, vertex2);
}

inline bool TopExp_CommonVertex(const TopoDS_Edge &edge1, const TopoDS_Edge &edge2, TopoDS_Vertex &vertex) {
  return TopExp::CommonVertex(edge1, edge2, vertex);
}

inline std::unique_ptr<TopoDS_Face> BRepIntCurveSurface_Inter_face(const BRepIntCurveSurface_Inter &intersector) {
  return std::unique_ptr<TopoDS_Face>(new TopoDS_Face(intersector.Face()));
}

inline std::unique_ptr<gp_Pnt> BRepIntCurveSurface_Inter_point(const BRepIntCurveSurface_Inter &intersector) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(intersector.Pnt()));
}

// BRepFeat
inline std::unique_ptr<BRepFeat_MakeCylindricalHole> BRepFeat_MakeCylindricalHole_ctor() {
  return std::unique_ptr<BRepFeat_MakeCylindricalHole>(new BRepFeat_MakeCylindricalHole());
}

// Data Import
inline IFSelect_ReturnStatus read_step(STEPControl_Reader &reader, rust::String theFileName) {
  return reader.ReadFile(theFileName.c_str());
}

inline IFSelect_ReturnStatus read_iges(IGESControl_Reader &reader, rust::String theFileName) {
  return reader.ReadFile(theFileName.c_str());
}

inline std::unique_ptr<TopoDS_Shape> one_shape_step(const STEPControl_Reader &reader) {
  return std::unique_ptr<TopoDS_Shape>(new TopoDS_Shape(reader.OneShape()));
}

inline std::unique_ptr<TopoDS_Shape> one_shape_iges(const IGESControl_Reader &reader) {
  return std::unique_ptr<TopoDS_Shape>(new TopoDS_Shape(reader.OneShape()));
}

// Data Export
inline IFSelect_ReturnStatus transfer_shape(STEPControl_Writer &writer, const TopoDS_Shape &theShape) {
  return writer.Transfer(theShape, STEPControl_AsIs);
}

inline void compute_model(IGESControl_Writer &writer) { writer.ComputeModel(); }

inline bool add_shape(IGESControl_Writer &writer, const TopoDS_Shape &theShape) { return writer.AddShape(theShape); }

inline IFSelect_ReturnStatus write_step(STEPControl_Writer &writer, rust::String theFileName) {
  return writer.Write(theFileName.c_str());
}

inline bool write_iges(IGESControl_Writer &writer, rust::String theFileName) {
  return writer.Write(theFileName.c_str());
}

inline bool write_stl(StlAPI_Writer &writer, const TopoDS_Shape &theShape, rust::String theFileName) {
  return writer.Write(theShape, theFileName.c_str());
}

inline std::unique_ptr<gp_Dir> Poly_Triangulation_Normal(const Poly_Triangulation &triangulation,
                                                         const Standard_Integer index) {
  return std::unique_ptr<gp_Dir>(new gp_Dir(triangulation.Normal(index)));
}

inline std::unique_ptr<gp_Pnt> Poly_Triangulation_Node(const Poly_Triangulation &triangulation,
                                                       const Standard_Integer index) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(triangulation.Node(index)));
}

inline std::unique_ptr<gp_Pnt2d> Poly_Triangulation_UV(const Poly_Triangulation &triangulation,
                                                       const Standard_Integer index) {
  return std::unique_ptr<gp_Pnt2d>(new gp_Pnt2d(triangulation.UVNode(index)));
}

inline void compute_normals(const TopoDS_Face &face, const Handle(Poly_Triangulation) & triangulation) {
  BRepLib_ToolTriangulatedShape::ComputeNormals(face, triangulation);
}

// Shape Properties
inline std::unique_ptr<gp_Pnt> GProp_GProps_CentreOfMass(const GProp_GProps &props) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(props.CentreOfMass()));
}

inline void BRepGProp_LinearProperties(const TopoDS_Shape &shape, GProp_GProps &props) {
  BRepGProp::LinearProperties(shape, props);
}

inline void BRepGProp_SurfaceProperties(const TopoDS_Shape &shape, GProp_GProps &props) {
  BRepGProp::SurfaceProperties(shape, props);
}

inline void BRepGProp_VolumeProperties(const TopoDS_Shape &shape, GProp_GProps &props) {
  BRepGProp::VolumeProperties(shape, props);
}

// Fillets
inline std::unique_ptr<TopoDS_Edge> BRepFilletAPI_MakeFillet2d_add_fillet(BRepFilletAPI_MakeFillet2d &make_fillet,
                                                                          const TopoDS_Vertex &vertex,
                                                                          Standard_Real radius) {
  return std::unique_ptr<TopoDS_Edge>(new TopoDS_Edge(make_fillet.AddFillet(vertex, radius)));
}

// Chamfers
inline std::unique_ptr<TopoDS_Edge>
BRepFilletAPI_MakeFillet2d_add_chamfer(BRepFilletAPI_MakeFillet2d &make_fillet, const TopoDS_Edge &edge1,
                                       const TopoDS_Edge &edge2, const Standard_Real dist1, const Standard_Real dist2) {
  return std::unique_ptr<TopoDS_Edge>(new TopoDS_Edge(make_fillet.AddChamfer(edge1, edge2, dist1, dist2)));
}

inline std::unique_ptr<TopoDS_Edge>
BRepFilletAPI_MakeFillet2d_add_chamfer_angle(BRepFilletAPI_MakeFillet2d &make_fillet, const TopoDS_Edge &edge,
                                             const TopoDS_Vertex &vertex, const Standard_Real dist,
                                             const Standard_Real angle) {
  return std::unique_ptr<TopoDS_Edge>(new TopoDS_Edge(make_fillet.AddChamfer(edge, vertex, dist, angle)));
}

// BRepTools
inline std::unique_ptr<TopoDS_Wire> outer_wire(const TopoDS_Face &face) {
  return std::unique_ptr<TopoDS_Wire>(new TopoDS_Wire(BRepTools::OuterWire(face)));
}

inline void BRepTools_UVBounds(const TopoDS_Face &face, double &umin, double &umax, double &vmin, double &vmax) {
  BRepTools::UVBounds(face, umin, umax, vmin, vmax);
}

// Collections
inline void map_shapes(const TopoDS_Shape &S, const TopAbs_ShapeEnum T, TopTools_IndexedMapOfShape &M) {
  TopExp::MapShapes(S, T, M);
}

inline void map_shapes_and_ancestors(const TopoDS_Shape &S, const TopAbs_ShapeEnum TS, const TopAbs_ShapeEnum TA,
                                     TopTools_IndexedDataMapOfShapeListOfShape &M) {
  TopExp::MapShapesAndAncestors(S, TS, TA, M);
}

inline void map_shapes_and_unique_ancestors(const TopoDS_Shape &S, const TopAbs_ShapeEnum TS, const TopAbs_ShapeEnum TA,
                                            TopTools_IndexedDataMapOfShapeListOfShape &M) {
  TopExp::MapShapesAndUniqueAncestors(S, TS, TA, M);
}

inline std::unique_ptr<gp_Dir> TColgp_Array1OfDir_Value(const TColgp_Array1OfDir &array, Standard_Integer index) {
  return std::unique_ptr<gp_Dir>(new gp_Dir(array.Value(index)));
}

inline std::unique_ptr<gp_Pnt2d> TColgp_Array1OfPnt2d_Value(const TColgp_Array1OfPnt2d &array, Standard_Integer index) {
  return std::unique_ptr<gp_Pnt2d>(new gp_Pnt2d(array.Value(index)));
}

inline std::unique_ptr<gp_Pnt> TColgp_HArray1OfPnt_Value(const TColgp_HArray1OfPnt &array, Standard_Integer index) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(array.Value(index)));
}

inline void connect_edges_to_wires(HandleTopTools_HSequenceOfShape &edges, const Standard_Real toler,
                                   const Standard_Boolean shared, HandleTopTools_HSequenceOfShape &wires) {
  ShapeAnalysis_FreeBounds::ConnectEdgesToWires(edges, toler, shared, wires);
}

inline std::unique_ptr<HandleTopTools_HSequenceOfShape> new_HandleTopTools_HSequenceOfShape() {
  auto sequence = new TopTools_HSequenceOfShape();
  auto handle = new opencascade::handle<TopTools_HSequenceOfShape>(sequence);

  return std::unique_ptr<HandleTopTools_HSequenceOfShape>(handle);
}

inline void TopTools_HSequenceOfShape_append(HandleTopTools_HSequenceOfShape &handle, const TopoDS_Shape &shape) {
  handle->Append(shape);
}

inline Standard_Integer TopTools_HSequenceOfShape_length(const HandleTopTools_HSequenceOfShape &handle) {
  return handle->Length();
}

inline const TopoDS_Shape &TopTools_HSequenceOfShape_value(const HandleTopTools_HSequenceOfShape &handle,
                                                           Standard_Integer index) {
  return handle->Value(index);
}

// BRep Algo API
inline std::unique_ptr<BRepAlgoAPI_BuilderAlgo>
cast_section_to_builderalgo(std::unique_ptr<BRepAlgoAPI_Section> section) {
  return section;
}
// namespace BRepAlgoAPI

// Bnd_Box
inline std::unique_ptr<Bnd_Box> Bnd_Box_ctor() { return std::unique_ptr<Bnd_Box>(new Bnd_Box()); }
inline std::unique_ptr<gp_Pnt> Bnd_Box_CornerMin(const Bnd_Box &box) {
  auto p = box.CornerMin();
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(p));
}
inline std::unique_ptr<gp_Pnt> Bnd_Box_CornerMax(const Bnd_Box &box) {
  auto p = box.CornerMax();
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(p));
}

// BRepBndLib
inline void BRepBndLib_Add(const TopoDS_Shape &shape, Bnd_Box &box, const Standard_Boolean useTriangulation) {
  BRepBndLib::Add(shape, box, useTriangulation);
}

// BRepAdaptor_Surface
inline std::unique_ptr<BRepAdaptor_Surface> BRepAdaptor_Surface_ctor(const TopoDS_Face &face, bool restriction) {
  return std::unique_ptr<BRepAdaptor_Surface>(new BRepAdaptor_Surface(face, restriction));
}

// Surface type extractors - these throw Standard_NoSuchObject if type doesn't match
// Wrapped with try/catch to prevent exceptions crossing FFI boundary
inline std::unique_ptr<gp_Pln> BRepAdaptor_Surface_Plane(const BRepAdaptor_Surface &surface) {
  try {
    return std::unique_ptr<gp_Pln>(new gp_Pln(surface.Plane()));
  } catch (const Standard_Failure &e) {
    get_occt_last_error_ref() = e.GetMessageString();
    return nullptr;
  }
}

inline std::unique_ptr<gp_Cylinder> BRepAdaptor_Surface_Cylinder(const BRepAdaptor_Surface &surface) {
  try {
    return std::unique_ptr<gp_Cylinder>(new gp_Cylinder(surface.Cylinder()));
  } catch (const Standard_Failure &e) {
    get_occt_last_error_ref() = e.GetMessageString();
    return nullptr;
  }
}

inline std::unique_ptr<gp_Cone> BRepAdaptor_Surface_Cone(const BRepAdaptor_Surface &surface) {
  try {
    return std::unique_ptr<gp_Cone>(new gp_Cone(surface.Cone()));
  } catch (const Standard_Failure &e) {
    get_occt_last_error_ref() = e.GetMessageString();
    return nullptr;
  }
}

// BRepAdaptor_Surface D1 evaluation - returns point and partial derivatives at (u,v)
// We provide separate functions for each output since cxx doesn't support out parameters
inline void BRepAdaptor_Surface_D1(
    const BRepAdaptor_Surface &surface,
    double u, double v,
    gp_Pnt &p, gp_Vec &d1u, gp_Vec &d1v
) {
  surface.D1(u, v, p, d1u, d1v);
}

// Convenience wrappers that return each D1 component as a UniquePtr
inline std::unique_ptr<gp_Pnt> BRepAdaptor_Surface_D1_Point(
    const BRepAdaptor_Surface &surface, double u, double v
) {
  gp_Pnt p;
  gp_Vec d1u, d1v;
  surface.D1(u, v, p, d1u, d1v);
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(p));
}

inline std::unique_ptr<gp_Vec> BRepAdaptor_Surface_D1U(
    const BRepAdaptor_Surface &surface, double u, double v
) {
  gp_Pnt p;
  gp_Vec d1u, d1v;
  surface.D1(u, v, p, d1u, d1v);
  return std::unique_ptr<gp_Vec>(new gp_Vec(d1u));
}

inline std::unique_ptr<gp_Vec> BRepAdaptor_Surface_D1V(
    const BRepAdaptor_Surface &surface, double u, double v
) {
  gp_Pnt p;
  gp_Vec d1u, d1v;
  surface.D1(u, v, p, d1u, d1v);
  return std::unique_ptr<gp_Vec>(new gp_Vec(d1v));
}

// gp_Pln
inline std::unique_ptr<gp_Pln> gp_Pln_ctor(const gp_Pnt &origin, const gp_Dir &normal) {
  return std::unique_ptr<gp_Pln>(new gp_Pln(origin, normal));
}

inline std::unique_ptr<gp_Pnt> gp_Pln_Location(const gp_Pln &plane) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(plane.Location()));
}

inline std::unique_ptr<gp_Ax1> gp_Pln_Axis(const gp_Pln &plane) {
  return std::unique_ptr<gp_Ax1>(new gp_Ax1(plane.Axis()));
}

inline std::unique_ptr<gp_Ax3> gp_Pln_Position(const gp_Pln &plane) {
  return std::unique_ptr<gp_Ax3>(new gp_Ax3(plane.Position()));
}

// gp_Cylinder
inline std::unique_ptr<gp_Ax1> gp_Cylinder_Axis(const gp_Cylinder &cylinder) {
  return std::unique_ptr<gp_Ax1>(new gp_Ax1(cylinder.Axis()));
}

inline std::unique_ptr<gp_Pnt> gp_Cylinder_Location(const gp_Cylinder &cylinder) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(cylinder.Location()));
}

inline std::unique_ptr<gp_Ax3> gp_Cylinder_Position(const gp_Cylinder &cylinder) {
  return std::unique_ptr<gp_Ax3>(new gp_Ax3(cylinder.Position()));
}

// gp_Cone
inline std::unique_ptr<gp_Pnt> gp_Cone_Apex(const gp_Cone &cone) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(cone.Apex()));
}

inline std::unique_ptr<gp_Ax1> gp_Cone_Axis(const gp_Cone &cone) {
  return std::unique_ptr<gp_Ax1>(new gp_Ax1(cone.Axis()));
}

inline std::unique_ptr<gp_Pnt> gp_Cone_Location(const gp_Cone &cone) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(cone.Location()));
}

inline std::unique_ptr<gp_Ax3> gp_Cone_Position(const gp_Cone &cone) {
  return std::unique_ptr<gp_Ax3>(new gp_Ax3(cone.Position()));
}

// gp_Sphere - Spherical surface (throws Standard_NoSuchObject if not a sphere)
inline std::unique_ptr<gp_Sphere> BRepAdaptor_Surface_Sphere(const BRepAdaptor_Surface &surface) {
  try {
    return std::unique_ptr<gp_Sphere>(new gp_Sphere(surface.Sphere()));
  } catch (const Standard_Failure &e) {
    get_occt_last_error_ref() = e.GetMessageString();
    return nullptr;
  }
}

inline std::unique_ptr<gp_Pnt> gp_Sphere_Location(const gp_Sphere &sphere) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(sphere.Location()));
}

inline std::unique_ptr<gp_Ax3> gp_Sphere_Position(const gp_Sphere &sphere) {
  return std::unique_ptr<gp_Ax3>(new gp_Ax3(sphere.Position()));
}

// gp_Torus - Toroidal surface (throws Standard_NoSuchObject if not a torus)
inline std::unique_ptr<gp_Torus> BRepAdaptor_Surface_Torus(const BRepAdaptor_Surface &surface) {
  try {
    return std::unique_ptr<gp_Torus>(new gp_Torus(surface.Torus()));
  } catch (const Standard_Failure &e) {
    get_occt_last_error_ref() = e.GetMessageString();
    return nullptr;
  }
}

inline std::unique_ptr<gp_Pnt> gp_Torus_Location(const gp_Torus &torus) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(torus.Location()));
}

inline std::unique_ptr<gp_Ax1> gp_Torus_Axis(const gp_Torus &torus) {
  return std::unique_ptr<gp_Ax1>(new gp_Ax1(torus.Axis()));
}

inline std::unique_ptr<gp_Ax3> gp_Torus_Position(const gp_Torus &torus) {
  return std::unique_ptr<gp_Ax3>(new gp_Ax3(torus.Position()));
}

// BRepAdaptor_Surface::Direction - for extrusion/revolution surfaces
inline std::unique_ptr<gp_Dir> BRepAdaptor_Surface_Direction(const BRepAdaptor_Surface &surface) {
  return std::unique_ptr<gp_Dir>(new gp_Dir(surface.Direction()));
}

// gp_Ax3 methods
inline std::unique_ptr<gp_Pnt> gp_Ax3_Location(const gp_Ax3 &axis) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(axis.Location()));
}

inline std::unique_ptr<gp_Dir> gp_Ax3_Direction(const gp_Ax3 &axis) {
  return std::unique_ptr<gp_Dir>(new gp_Dir(axis.Direction()));
}

inline std::unique_ptr<gp_Dir> gp_Ax3_XDirection(const gp_Ax3 &axis) {
  return std::unique_ptr<gp_Dir>(new gp_Dir(axis.XDirection()));
}

inline std::unique_ptr<gp_Dir> gp_Ax3_YDirection(const gp_Ax3 &axis) {
  return std::unique_ptr<gp_Dir>(new gp_Dir(axis.YDirection()));
}

inline std::unique_ptr<gp_Ax1> gp_Ax3_Axis(const gp_Ax3 &axis) {
  return std::unique_ptr<gp_Ax1>(new gp_Ax1(axis.Axis()));
}

// BRepLProp_SLProps - Surface local properties
inline std::unique_ptr<BRepLProp_SLProps> BRepLProp_SLProps_ctor(
    const BRepAdaptor_Surface &surface, double u, double v, int n, double resolution
) {
  return std::unique_ptr<BRepLProp_SLProps>(new BRepLProp_SLProps(surface, u, v, n, resolution));
}

inline bool BRepLProp_SLProps_IsNormalDefined(BRepLProp_SLProps &props) {
  return props.IsNormalDefined();
}

inline std::unique_ptr<gp_Dir> BRepLProp_SLProps_Normal(BRepLProp_SLProps &props) {
  return std::unique_ptr<gp_Dir>(new gp_Dir(props.Normal()));
}

inline std::unique_ptr<gp_Pnt> BRepLProp_SLProps_Value(BRepLProp_SLProps &props) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(props.Value()));
}

inline std::unique_ptr<gp_Vec> BRepLProp_SLProps_D1U(BRepLProp_SLProps &props) {
  return std::unique_ptr<gp_Vec>(new gp_Vec(props.D1U()));
}

inline std::unique_ptr<gp_Vec> BRepLProp_SLProps_D1V(BRepLProp_SLProps &props) {
  return std::unique_ptr<gp_Vec>(new gp_Vec(props.D1V()));
}

// BRepAdaptor_Curve2d - 2D curve on a face surface
inline std::unique_ptr<BRepAdaptor_Curve2d> BRepAdaptor_Curve2d_ctor(
    const TopoDS_Edge &edge, const TopoDS_Face &face
) {
  return std::unique_ptr<BRepAdaptor_Curve2d>(new BRepAdaptor_Curve2d(edge, face));
}

inline std::unique_ptr<gp_Pnt2d> BRepAdaptor_Curve2d_Value(const BRepAdaptor_Curve2d &curve, double u) {
  return std::unique_ptr<gp_Pnt2d>(new gp_Pnt2d(curve.Value(u)));
}

// gp_Lin additional methods
inline std::unique_ptr<gp_Pnt> gp_Lin_Location(const gp_Lin &line) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(line.Location()));
}

inline std::unique_ptr<gp_Dir> gp_Lin_Direction(const gp_Lin &line) {
  return std::unique_ptr<gp_Dir>(new gp_Dir(line.Direction()));
}

// BRepBuilderAPI_Sewing - Sew faces into shells
inline std::unique_ptr<BRepBuilderAPI_Sewing> BRepBuilderAPI_Sewing_ctor(double tolerance) {
  return std::unique_ptr<BRepBuilderAPI_Sewing>(new BRepBuilderAPI_Sewing(tolerance));
}

inline std::unique_ptr<TopoDS_Shape> BRepBuilderAPI_Sewing_SewedShape(const BRepBuilderAPI_Sewing &sewing) {
  return std::unique_ptr<TopoDS_Shape>(new TopoDS_Shape(sewing.SewedShape()));
}

// ShapeFix_Shape - Repair invalid geometry
inline std::unique_ptr<ShapeFix_Shape> ShapeFix_Shape_ctor(const TopoDS_Shape &shape) {
  return std::unique_ptr<ShapeFix_Shape>(new ShapeFix_Shape(shape));
}

inline std::unique_ptr<TopoDS_Shape> ShapeFix_Shape_Shape(const ShapeFix_Shape &fixer) {
  return std::unique_ptr<TopoDS_Shape>(new TopoDS_Shape(fixer.Shape()));
}

// GCPnts_AbscissaPoint - Find points at specific arc lengths on curves
inline std::unique_ptr<GCPnts_AbscissaPoint> GCPnts_AbscissaPoint_ctor(
    const BRepAdaptor_Curve &curve, double abscissa, double u0
) {
  return std::unique_ptr<GCPnts_AbscissaPoint>(new GCPnts_AbscissaPoint(curve, abscissa, u0));
}

inline double GCPnts_AbscissaPoint_Length(const BRepAdaptor_Curve &curve) {
  return GCPnts_AbscissaPoint::Length(curve);
}

inline double GCPnts_AbscissaPoint_Length_bounds(
    const BRepAdaptor_Curve &curve, double u1, double u2
) {
  return GCPnts_AbscissaPoint::Length(curve, u1, u2);
}

// TColStd_Array1OfReal - Array of real numbers (for knots, weights)
inline std::unique_ptr<TColStd_Array1OfReal> TColStd_Array1OfReal_ctor(
    Standard_Integer lower, Standard_Integer upper
) {
  return std::unique_ptr<TColStd_Array1OfReal>(new TColStd_Array1OfReal(lower, upper));
}

inline Standard_Real TColStd_Array1OfReal_Value(
    const TColStd_Array1OfReal &array, Standard_Integer index
) {
  return array.Value(index);
}

inline void TColStd_Array1OfReal_SetValue(
    TColStd_Array1OfReal &array, Standard_Integer index, Standard_Real value
) {
  array.SetValue(index, value);
}

// TColStd_Array1OfInteger - Array of integers (for multiplicities)
inline std::unique_ptr<TColStd_Array1OfInteger> TColStd_Array1OfInteger_ctor(
    Standard_Integer lower, Standard_Integer upper
) {
  return std::unique_ptr<TColStd_Array1OfInteger>(new TColStd_Array1OfInteger(lower, upper));
}

inline Standard_Integer TColStd_Array1OfInteger_Value(
    const TColStd_Array1OfInteger &array, Standard_Integer index
) {
  return array.Value(index);
}

inline void TColStd_Array1OfInteger_SetValue(
    TColStd_Array1OfInteger &array, Standard_Integer index, Standard_Integer value
) {
  array.SetValue(index, value);
}

// TColgp_Array1OfPnt - Array of points (for B-spline poles)
inline std::unique_ptr<TColgp_Array1OfPnt> TColgp_Array1OfPnt_ctor(
    Standard_Integer lower, Standard_Integer upper
) {
  return std::unique_ptr<TColgp_Array1OfPnt>(new TColgp_Array1OfPnt(lower, upper));
}

inline std::unique_ptr<gp_Pnt> TColgp_Array1OfPnt_Value(
    const TColgp_Array1OfPnt &array, Standard_Integer index
) {
  return std::unique_ptr<gp_Pnt>(new gp_Pnt(array.Value(index)));
}

inline void TColgp_Array1OfPnt_SetValue(
    TColgp_Array1OfPnt &array, Standard_Integer index, const gp_Pnt &value
) {
  array.SetValue(index, value);
}

// Geom_BSplineCurve - Non-rational B-spline curve constructor
// Throws Standard_ConstructionError if parameters are invalid
inline std::unique_ptr<Geom_BSplineCurve> Geom_BSplineCurve_ctor(
    const TColgp_Array1OfPnt &poles,
    const TColStd_Array1OfReal &knots,
    const TColStd_Array1OfInteger &multiplicities,
    Standard_Integer degree,
    Standard_Boolean periodic
) {
  try {
    return std::unique_ptr<Geom_BSplineCurve>(
      new Geom_BSplineCurve(poles, knots, multiplicities, degree, periodic)
    );
  } catch (const Standard_Failure &e) {
    get_occt_last_error_ref() = e.GetMessageString();
    return nullptr;
  }
}

// Geom_BSplineCurve - Rational B-spline (NURBS) curve constructor
// Throws Standard_ConstructionError if parameters are invalid
inline std::unique_ptr<Geom_BSplineCurve> Geom_BSplineCurve_ctor_weighted(
    const TColgp_Array1OfPnt &poles,
    const TColStd_Array1OfReal &weights,
    const TColStd_Array1OfReal &knots,
    const TColStd_Array1OfInteger &multiplicities,
    Standard_Integer degree,
    Standard_Boolean periodic
) {
  try {
    return std::unique_ptr<Geom_BSplineCurve>(
      new Geom_BSplineCurve(poles, weights, knots, multiplicities, degree, periodic, Standard_True)
    );
  } catch (const Standard_Failure &e) {
    get_occt_last_error_ref() = e.GetMessageString();
    return nullptr;
  }
}

// Convert Geom_BSplineCurve to handle
inline std::unique_ptr<HandleGeomBSplineCurve> Geom_BSplineCurve_to_handle(
    std::unique_ptr<Geom_BSplineCurve> curve
) {
  return std::unique_ptr<HandleGeomBSplineCurve>(
    new opencascade::handle<Geom_BSplineCurve>(curve.release())
  );
}

// BRepOffsetAPI_MakeOffsetShape - Offset a shape in 3D
inline std::unique_ptr<BRepOffsetAPI_MakeOffsetShape> BRepOffsetAPI_MakeOffsetShape_ctor() {
  return std::unique_ptr<BRepOffsetAPI_MakeOffsetShape>(new BRepOffsetAPI_MakeOffsetShape());
}

inline void BRepOffsetAPI_MakeOffsetShape_PerformByJoin(
    BRepOffsetAPI_MakeOffsetShape &maker,
    const TopoDS_Shape &shape,
    Standard_Real offset,
    Standard_Real tolerance,
    BRepOffset_Mode mode,
    Standard_Boolean intersection,
    Standard_Boolean selfInter,
    GeomAbs_JoinType join,
    Standard_Boolean removeIntEdges
) {
  maker.PerformByJoin(shape, offset, tolerance, mode, intersection, selfInter, join, removeIntEdges);
}

// Geom_Plane - Create a plane from gp_Ax3
inline std::unique_ptr<Geom_Plane> Geom_Plane_ctor(const gp_Ax3 &axis) {
  return std::unique_ptr<Geom_Plane>(new Geom_Plane(axis));
}

inline std::unique_ptr<HandleGeomPlane> Geom_Plane_to_handle(std::unique_ptr<Geom_Plane> plane) {
  return std::unique_ptr<HandleGeomPlane>(new opencascade::handle<Geom_Plane>(plane.release()));
}

inline std::unique_ptr<HandleGeomSurface> HandleGeomPlane_to_HandleGeomSurface(
    const HandleGeomPlane &plane
) {
  return std::unique_ptr<HandleGeomSurface>(new opencascade::handle<Geom_Surface>(plane));
}

// gp_Ax3 - Create XOY coordinate system (Z=0 plane)
inline std::unique_ptr<gp_Ax3> gp_Ax3_XOY() {
  return std::unique_ptr<gp_Ax3>(new gp_Ax3(gp::XOY()));
}

// gp_Lin2d - 2D line
inline std::unique_ptr<gp_Pnt2d> gp_Lin2d_Location(const gp_Lin2d &line) {
  return std::unique_ptr<gp_Pnt2d>(new gp_Pnt2d(line.Location()));
}

inline std::unique_ptr<gp_Dir2d> gp_Lin2d_Direction(const gp_Lin2d &line) {
  return std::unique_ptr<gp_Dir2d>(new gp_Dir2d(line.Direction()));
}

// gp_Circ2d - 2D circle
inline std::unique_ptr<gp_Pnt2d> gp_Circ2d_Location(const gp_Circ2d &circ) {
  return std::unique_ptr<gp_Pnt2d>(new gp_Pnt2d(circ.Location()));
}

inline Standard_Real gp_Circ2d_Radius(const gp_Circ2d &circ) {
  return circ.Radius();
}

// Geom2d_Curve type detection - returns the type name string
inline rust::String Geom2d_Curve_DynamicType(const HandleGeom2d_Curve &curve) {
  if (curve.IsNull()) return "";
  return std::string(curve->DynamicType()->Name());
}

// Geom2d_Line - downcast and accessors
inline std::unique_ptr<HandleGeom2d_Line> HandleGeom2d_Curve_to_Line(
    const HandleGeom2d_Curve &curve
) {
  HandleGeom2d_Line line = opencascade::handle<Geom2d_Line>::DownCast(curve);
  return std::unique_ptr<HandleGeom2d_Line>(new opencascade::handle<Geom2d_Line>(line));
}

inline std::unique_ptr<gp_Lin2d> Geom2d_Line_Lin2d(const HandleGeom2d_Line &line) {
  return std::unique_ptr<gp_Lin2d>(new gp_Lin2d(line->Lin2d()));
}

// Geom2d_Circle - downcast and accessors
inline std::unique_ptr<HandleGeom2d_Circle> HandleGeom2d_Curve_to_Circle(
    const HandleGeom2d_Curve &curve
) {
  HandleGeom2d_Circle circle = opencascade::handle<Geom2d_Circle>::DownCast(curve);
  return std::unique_ptr<HandleGeom2d_Circle>(new opencascade::handle<Geom2d_Circle>(circle));
}

inline std::unique_ptr<gp_Circ2d> Geom2d_Circle_Circ2d(const HandleGeom2d_Circle &circle) {
  return std::unique_ptr<gp_Circ2d>(new gp_Circ2d(circle->Circ2d()));
}

// Geom2d_BSplineCurve - downcast and accessors
inline std::unique_ptr<HandleGeom2d_BSplineCurve> HandleGeom2d_Curve_to_BSplineCurve(
    const HandleGeom2d_Curve &curve
) {
  HandleGeom2d_BSplineCurve bspline = opencascade::handle<Geom2d_BSplineCurve>::DownCast(curve);
  return std::unique_ptr<HandleGeom2d_BSplineCurve>(
    new opencascade::handle<Geom2d_BSplineCurve>(bspline)
  );
}

inline std::unique_ptr<HandleGeom2d_Curve> HandleGeom2d_BSplineCurve_to_HandleGeom2d_Curve(
    const HandleGeom2d_BSplineCurve &curve
) {
  return std::unique_ptr<HandleGeom2d_Curve>(new opencascade::handle<Geom2d_Curve>(curve));
}

inline Standard_Integer Geom2d_BSplineCurve_Degree(const HandleGeom2d_BSplineCurve &curve) {
  return curve->Degree();
}

inline Standard_Integer Geom2d_BSplineCurve_NbPoles(const HandleGeom2d_BSplineCurve &curve) {
  return curve->NbPoles();
}

inline Standard_Integer Geom2d_BSplineCurve_NbKnots(const HandleGeom2d_BSplineCurve &curve) {
  return curve->NbKnots();
}

inline std::unique_ptr<gp_Pnt2d> Geom2d_BSplineCurve_Pole(
    const HandleGeom2d_BSplineCurve &curve, Standard_Integer index
) {
  return std::unique_ptr<gp_Pnt2d>(new gp_Pnt2d(curve->Pole(index)));
}

inline Standard_Real Geom2d_BSplineCurve_Knot(
    const HandleGeom2d_BSplineCurve &curve, Standard_Integer index
) {
  return curve->Knot(index);
}

inline Standard_Integer Geom2d_BSplineCurve_Multiplicity(
    const HandleGeom2d_BSplineCurve &curve, Standard_Integer index
) {
  return curve->Multiplicity(index);
}

inline Standard_Real Geom2d_BSplineCurve_Weight(
    const HandleGeom2d_BSplineCurve &curve, Standard_Integer index
) {
  return curve->Weight(index);
}

inline Standard_Boolean Geom2d_BSplineCurve_IsRational(const HandleGeom2d_BSplineCurve &curve) {
  return curve->IsRational();
}

inline Standard_Boolean Geom2d_BSplineCurve_IsPeriodic(const HandleGeom2d_BSplineCurve &curve) {
  return curve->IsPeriodic();
}

inline void Geom2d_BSplineCurve_SetPole(
    const HandleGeom2d_BSplineCurve &curve, Standard_Integer index, const gp_Pnt2d &pole
) {
  curve->SetPole(index, pole);
}

// Geom_Line - Create a line from point and direction
inline std::unique_ptr<Geom_Line> Geom_Line_ctor(const gp_Pnt &point, const gp_Dir &dir) {
  return std::unique_ptr<Geom_Line>(new Geom_Line(point, dir));
}

inline std::unique_ptr<HandleGeomLine> Geom_Line_to_handle(std::unique_ptr<Geom_Line> line) {
  return std::unique_ptr<HandleGeomLine>(new opencascade::handle<Geom_Line>(line.release()));
}

inline std::unique_ptr<HandleGeomCurve> HandleGeomLine_to_HandleGeomCurve(
    const HandleGeomLine &line
) {
  return std::unique_ptr<HandleGeomCurve>(new opencascade::handle<Geom_Curve>(line));
}

// BRepBuilderAPI_MakeFace - Add an inner wire (hole)
inline void BRepBuilderAPI_MakeFace_Add(BRepBuilderAPI_MakeFace &maker, const TopoDS_Wire &wire) {
  maker.Add(wire);
}

// BRepBuilderAPI_MakeEdge - Create edge from curve with parameter bounds
inline std::unique_ptr<BRepBuilderAPI_MakeEdge> BRepBuilderAPI_MakeEdge_HandleGeomCurve_with_params(
    const HandleGeomCurve &curve, Standard_Real p1, Standard_Real p2
) {
  return std::unique_ptr<BRepBuilderAPI_MakeEdge>(new BRepBuilderAPI_MakeEdge(curve, p1, p2));
}

// Geom2d_Line - Direct location and direction accessors
inline std::unique_ptr<gp_Pnt2d> Geom2d_Line_Location(const HandleGeom2d_Line &line) {
  return std::unique_ptr<gp_Pnt2d>(new gp_Pnt2d(line->Location()));
}

inline std::unique_ptr<gp_Dir2d> Geom2d_Line_Direction(const HandleGeom2d_Line &line) {
  return std::unique_ptr<gp_Dir2d>(new gp_Dir2d(line->Direction()));
}

// Geom2dConvert - Convert any 2D curve to B-spline
// Throws Standard_DomainError for infinite curves, Standard_ConstructionError for unsupported types
inline std::unique_ptr<HandleGeom2d_BSplineCurve> Geom2dConvert_CurveToBSplineCurve(
    const HandleGeom2d_Curve &curve
) {
  try {
    opencascade::handle<Geom2d_BSplineCurve> bspline = Geom2dConvert::CurveToBSplineCurve(curve);
    return std::unique_ptr<HandleGeom2d_BSplineCurve>(
      new opencascade::handle<Geom2d_BSplineCurve>(bspline)
    );
  } catch (const Standard_Failure &e) {
    get_occt_last_error_ref() = e.GetMessageString();
    return nullptr;
  }
}

// Geom_BSplineCurve - Create from rust vectors (copies data into OCCT arrays)
// Throws Standard_ConstructionError if parameters are invalid
inline std::unique_ptr<HandleGeomBSplineCurve> Geom_BSplineCurve_from_vectors(
    rust::Slice<const double> pole_coords,  // flattened [x1,y1,z1, x2,y2,z2, ...]
    rust::Slice<const double> weights,      // empty slice for non-rational
    rust::Slice<const double> knots,
    rust::Slice<const int> multiplicities,
    Standard_Integer degree,
    Standard_Boolean periodic
) {
  try {
    Standard_Integer nb_poles = pole_coords.size() / 3;
    Standard_Integer nb_knots = knots.size();

    // Build poles array
    TColgp_Array1OfPnt poles_array(1, nb_poles);
    for (Standard_Integer i = 0; i < nb_poles; i++) {
      poles_array.SetValue(i + 1, gp_Pnt(
        pole_coords[i * 3],
        pole_coords[i * 3 + 1],
        pole_coords[i * 3 + 2]
      ));
    }

    // Build knots array
    TColStd_Array1OfReal knots_array(1, nb_knots);
    for (Standard_Integer i = 0; i < nb_knots; i++) {
      knots_array.SetValue(i + 1, knots[i]);
    }

    // Build multiplicities array
    TColStd_Array1OfInteger mults_array(1, nb_knots);
    for (Standard_Integer i = 0; i < nb_knots; i++) {
      mults_array.SetValue(i + 1, multiplicities[i]);
    }

    opencascade::handle<Geom_BSplineCurve> curve;

    if (weights.empty()) {
      // Non-rational B-spline
      curve = new Geom_BSplineCurve(poles_array, knots_array, mults_array, degree, periodic);
    } else {
      // Rational B-spline (NURBS)
      TColStd_Array1OfReal weights_array(1, nb_poles);
      for (Standard_Integer i = 0; i < nb_poles; i++) {
        weights_array.SetValue(i + 1, weights[i]);
      }
      curve = new Geom_BSplineCurve(poles_array, weights_array, knots_array, mults_array, degree, periodic);
    }

    return std::unique_ptr<HandleGeomBSplineCurve>(new HandleGeomBSplineCurve(curve));
  } catch (const Standard_Failure &e) {
    get_occt_last_error_ref() = e.GetMessageString();
    return nullptr;
  }
}
