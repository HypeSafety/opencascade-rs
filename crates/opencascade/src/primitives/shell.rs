use cxx::UniquePtr;
use opencascade_sys::ffi;

use crate::primitives::Wire;
use crate::Error;

pub struct Shell {
    pub(crate) inner: UniquePtr<ffi::TopoDS_Shell>,
}

impl AsRef<Shell> for Shell {
    fn as_ref(&self) -> &Shell {
        self
    }
}

impl Shell {
    pub(crate) fn from_shell(shell: &ffi::TopoDS_Shell) -> Self {
        let inner = ffi::TopoDS_Shell_to_owned(shell);

        Self { inner }
    }

    pub fn loft<T: AsRef<Wire>>(wires: impl IntoIterator<Item = T>) -> Result<Self, Error> {
        let is_solid = false;
        let mut make_loft = ffi::BRepOffsetAPI_ThruSections_ctor(is_solid);

        for wire in wires.into_iter() {
            make_loft.pin_mut().AddWire(&wire.as_ref().inner);
        }

        // Set CheckCompatibility to `true` to avoid twisted results.
        make_loft.pin_mut().CheckCompatibility(true);
        make_loft.pin_mut().Build(&ffi::Message_ProgressRange_ctor());

        if !make_loft.IsDone() {
            return Err(Error::LoftFailed);
        }

        let shape = make_loft.pin_mut().Shape();
        let shell = ffi::TopoDS_cast_to_shell(shape);

        Ok(Self::from_shell(shell))
    }
}
