//! XDE/XCAF Assembly support for reading STEP files with preserved structure.
//!
//! This module provides the ability to read STEP files while preserving:
//! - Assembly hierarchy (parent-child relationships)
//! - Component names
//! - Instance transforms
//!
//! # Example
//!
//! ```no_run
//! use opencascade::assembly::AssemblyDocument;
//! use glam::DMat4;
//!
//! let doc = AssemblyDocument::read_step("assembly.step").unwrap();
//!
//! for root in doc.roots() {
//!     process_node(&root, DMat4::IDENTITY);
//! }
//!
//! fn process_node(node: &opencascade::assembly::AssemblyNode, parent_transform: DMat4) {
//!     let world_transform = parent_transform * node.transform();
//!     println!("Node: {} (assembly: {})", node.name(), node.is_assembly());
//!
//!     if node.is_assembly() {
//!         for child in node.children() {
//!             process_node(&child, world_transform);
//!         }
//!     } else {
//!         let shape = node.shape();
//!         // Use shape with world_transform...
//!     }
//! }
//! ```

use crate::{primitives::Shape, Error};
use cxx::UniquePtr;
use glam::DMat4;
use opencascade_sys::ffi;
use std::path::Path;

/// An XDE document containing assembly structure from a STEP file.
///
/// This preserves the assembly hierarchy, component names, and transforms
/// that would otherwise be lost when using `Shape::read_step()`.
pub struct AssemblyDocument {
    #[allow(dead_code)]
    doc: UniquePtr<ffi::HandleTDocStd_Document>,
    shape_tool: UniquePtr<ffi::HandleXCAFDoc_ShapeTool>,
}

impl AssemblyDocument {
    /// Read a STEP file preserving assembly structure.
    ///
    /// Unlike `Shape::read_step()`, this preserves:
    /// - Assembly hierarchy
    /// - Component names
    /// - Instance transforms
    ///
    /// # Errors
    ///
    /// Returns `Error::StepReadFailed` if the file cannot be read.
    pub fn read_step(path: impl AsRef<Path>) -> Result<Self, Error> {
        // Create a new XDE document
        let mut doc = ffi::TDocStd_Document_ctor("MDTV-XCAF".to_string());

        if ffi::HandleTDocStd_Document_IsNull(&doc) {
            return Err(Error::StepReadFailed);
        }

        // Get the shape tool BEFORE transfer (critical!)
        let main_label = ffi::TDocStd_Document_Main(&doc);
        let shape_tool = ffi::XCAFDoc_DocumentTool_ShapeTool(&main_label);

        if ffi::HandleXCAFDoc_ShapeTool_IsNull(&shape_tool) {
            return Err(Error::StepReadFailed);
        }

        // Create and configure the STEP CAF reader
        let mut reader = ffi::STEPCAFControl_Reader_ctor();

        // Enable reading names, colors, and layers
        ffi::STEPCAFControl_Reader_SetNameMode(reader.pin_mut(), true);
        ffi::STEPCAFControl_Reader_SetColorMode(reader.pin_mut(), true);
        ffi::STEPCAFControl_Reader_SetLayerMode(reader.pin_mut(), true);

        // Read the file
        let path_str = path.as_ref().to_string_lossy().to_string();
        let status = ffi::STEPCAFControl_Reader_ReadFile(reader.pin_mut(), path_str);

        if status != ffi::IFSelect_ReturnStatus::IFSelect_RetDone {
            return Err(Error::StepReadFailed);
        }

        // Transfer to the document
        let success = ffi::STEPCAFControl_Reader_Transfer(reader.pin_mut(), doc.pin_mut());

        if !success {
            return Err(Error::StepReadFailed);
        }

        Ok(Self { doc, shape_tool })
    }

    /// Get an iterator over root (top-level) shapes in the assembly.
    ///
    /// These are the "free shapes" - shapes not referenced by any other shape.
    pub fn roots(&self) -> impl Iterator<Item = AssemblyNode<'_>> + '_ {
        let mut labels = ffi::TDF_LabelSequence_ctor();
        ffi::XCAFDoc_ShapeTool_GetFreeShapes(&self.shape_tool, labels.pin_mut());

        let len = ffi::TDF_LabelSequence_Length(&labels);
        RootIterator {
            labels,
            shape_tool: &self.shape_tool,
            index: 1, // 1-based indexing!
            len,
        }
    }

    /// Collect all leaf shapes with their world transforms.
    ///
    /// This flattens the assembly tree, returning each leaf shape paired
    /// with its accumulated transform from the root.
    pub fn collect_leaf_shapes(&self) -> Vec<(Shape, DMat4)> {
        let mut result = Vec::new();

        for root in self.roots() {
            root.collect_leaf_shapes_recursive(DMat4::IDENTITY, &mut result);
        }

        result
    }
}

struct RootIterator<'a> {
    labels: UniquePtr<ffi::TDF_LabelSequence>,
    shape_tool: &'a UniquePtr<ffi::HandleXCAFDoc_ShapeTool>,
    index: i32,
    len: i32,
}

impl<'a> Iterator for RootIterator<'a> {
    type Item = AssemblyNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.len {
            return None;
        }

        let label = ffi::TDF_LabelSequence_Value(&self.labels, self.index);
        self.index += 1;

        Some(AssemblyNode {
            label,
            shape_tool: self.shape_tool,
        })
    }
}

/// A node in the assembly tree (either an assembly or a leaf part).
pub struct AssemblyNode<'a> {
    label: UniquePtr<ffi::TDF_Label>,
    shape_tool: &'a UniquePtr<ffi::HandleXCAFDoc_ShapeTool>,
}

impl<'a> AssemblyNode<'a> {
    /// Get the name of this node (from the STEP file).
    ///
    /// Returns an empty string if no name is defined.
    pub fn name(&self) -> String {
        ffi::TDF_Label_GetName(&self.label)
    }

    /// Check if this node is an assembly (has children).
    pub fn is_assembly(&self) -> bool {
        ffi::XCAFDoc_ShapeTool_IsAssembly(self.shape_tool, &self.label)
    }

    /// Check if this node is a component (instance reference).
    pub fn is_component(&self) -> bool {
        ffi::XCAFDoc_ShapeTool_IsComponent(self.shape_tool, &self.label)
    }

    /// Check if this node is a simple shape (not an assembly, not a reference).
    pub fn is_simple_shape(&self) -> bool {
        ffi::XCAFDoc_ShapeTool_IsSimpleShape(self.shape_tool, &self.label)
    }

    /// Get the local transform of this node relative to its parent.
    ///
    /// For components, this is the instance transform.
    /// For other nodes, this is typically identity.
    pub fn transform(&self) -> DMat4 {
        let location = ffi::XCAFDoc_ShapeTool_GetLocation(&self.label);

        if ffi::TopLoc_Location_IsIdentity(&location) {
            return DMat4::IDENTITY;
        }

        let trsf = ffi::TopLoc_Location_Transformation(&location);
        trsf_to_dmat4(&trsf)
    }

    /// Get the shape at this node (without applying any transform).
    ///
    /// For components, this resolves the reference to get the actual shape.
    pub fn shape(&self) -> Shape {
        // If this is a component (reference), get the referred shape
        if self.is_component() {
            let mut ref_label = ffi::TDF_Label_ctor();
            if ffi::XCAFDoc_ShapeTool_GetReferredShape(
                self.shape_tool,
                &self.label,
                ref_label.pin_mut(),
            ) {
                let inner = ffi::XCAFDoc_ShapeTool_GetShape(&ref_label);
                return Shape { inner };
            }
        }

        // Otherwise get the shape directly from this label
        let inner = ffi::XCAFDoc_ShapeTool_GetShape(&self.label);
        Shape { inner }
    }

    /// Get the shape with its local transform applied.
    ///
    /// Note: For complex transforms, you may want to use `shape()` and `transform()`
    /// separately and apply the transform yourself for more control.
    pub fn shape_transformed(&self) -> Shape {
        let transform = self.transform();

        if transform == DMat4::IDENTITY {
            return self.shape();
        }

        // Build a gp_Trsf from the DMat4
        let trsf = dmat4_to_trsf(transform);
        let shape = self.shape();
        shape.transform(&trsf)
    }

    /// Get an iterator over the children of this assembly node.
    ///
    /// Returns an empty iterator if this is not an assembly.
    pub fn children(&self) -> impl Iterator<Item = AssemblyNode<'a>> + '_ {
        let mut labels = ffi::TDF_LabelSequence_ctor();

        // Get direct children only (not recursive)
        ffi::XCAFDoc_ShapeTool_GetComponents(
            self.shape_tool,
            &self.label,
            labels.pin_mut(),
            false, // Don't get sub-children
        );

        let len = ffi::TDF_LabelSequence_Length(&labels);

        ChildIterator {
            labels,
            shape_tool: self.shape_tool,
            index: 1, // 1-based indexing!
            len,
        }
    }

    /// Recursively collect all leaf shapes with their accumulated transforms.
    fn collect_leaf_shapes_recursive(&self, parent_transform: DMat4, result: &mut Vec<(Shape, DMat4)>) {
        let world_transform = parent_transform * self.transform();

        if self.is_assembly() {
            for child in self.children() {
                child.collect_leaf_shapes_recursive(world_transform, result);
            }
        } else {
            let shape = self.shape();
            result.push((shape, world_transform));
        }
    }
}

struct ChildIterator<'a> {
    labels: UniquePtr<ffi::TDF_LabelSequence>,
    shape_tool: &'a UniquePtr<ffi::HandleXCAFDoc_ShapeTool>,
    index: i32,
    len: i32,
}

impl<'a> Iterator for ChildIterator<'a> {
    type Item = AssemblyNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.len {
            return None;
        }

        let label = ffi::TDF_LabelSequence_Value(&self.labels, self.index);
        self.index += 1;

        Some(AssemblyNode {
            label,
            shape_tool: self.shape_tool,
        })
    }
}

/// Convert an OpenCascade gp_Trsf to a glam DMat4.
fn trsf_to_dmat4(trsf: &UniquePtr<ffi::gp_Trsf>) -> DMat4 {
    // Get the 3x3 rotation/scale matrix (row-major)
    let rotation = ffi::gp_Trsf_VectorialPart(trsf);

    // Get the translation
    let translation = ffi::gp_Trsf_TranslationPart(trsf);
    let tx = translation.X();
    let ty = translation.Y();
    let tz = translation.Z();

    // Extract rotation values from the CxxVector
    // OCCT returns row-major: [R00, R01, R02, R10, R11, R12, R20, R21, R22]
    let r: Vec<f64> = rotation.iter().copied().collect();

    // Build the 4x4 matrix (column-major for glam)
    DMat4::from_cols_array(&[
        r[0], r[3], r[6], 0.0, // Column 0
        r[1], r[4], r[7], 0.0, // Column 1
        r[2], r[5], r[8], 0.0, // Column 2
        tx, ty, tz, 1.0,       // Column 3 (translation)
    ])
}

/// Convert a glam DMat4 to an OpenCascade gp_Trsf.
///
/// Note: gp_Trsf only supports affine transforms (rotation, translation, scale).
/// The bottom row of the DMat4 is expected to be [0, 0, 0, 1].
fn dmat4_to_trsf(mat: DMat4) -> UniquePtr<ffi::gp_Trsf> {
    let mut trsf = ffi::new_transform();

    // DMat4 is column-major, gp_Trsf::SetValues expects row-major
    // DMat4 layout: [col0.x, col0.y, col0.z, col0.w, col1.x, col1.y, ...]
    let cols = mat.to_cols_array();

    // Column 0: [cols[0], cols[1], cols[2], cols[3]]
    // Column 1: [cols[4], cols[5], cols[6], cols[7]]
    // Column 2: [cols[8], cols[9], cols[10], cols[11]]
    // Column 3: [cols[12], cols[13], cols[14], cols[15]] (translation + w)

    // gp_Trsf::SetValues takes row-major:
    // Row 1: a11, a12, a13, a14 -> cols[0], cols[4], cols[8], cols[12]
    // Row 2: a21, a22, a23, a24 -> cols[1], cols[5], cols[9], cols[13]
    // Row 3: a31, a32, a33, a34 -> cols[2], cols[6], cols[10], cols[14]

    ffi::gp_Trsf_SetValues(
        trsf.pin_mut(),
        cols[0], cols[4], cols[8], cols[12],   // Row 1
        cols[1], cols[5], cols[9], cols[13],   // Row 2
        cols[2], cols[6], cols[10], cols[14],  // Row 3
    );

    trsf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trsf_to_dmat4_identity() {
        // Test that identity transform converts correctly
        let location = ffi::TopLoc_Location_Identity();
        let trsf = ffi::TopLoc_Location_Transformation(&location);
        let mat = trsf_to_dmat4(&trsf);

        assert!((mat - DMat4::IDENTITY).abs_diff_eq(DMat4::ZERO, 1e-10));
    }
}
