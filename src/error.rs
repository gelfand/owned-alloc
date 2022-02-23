use core::alloc::{Layout, LayoutError as CoreLayoutError};

#[derive(Debug, Clone)]

pub struct AllocError {
    pub layout: Layout,
}
impl core::fmt::Display for AllocError {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Allocation error, size: {:#?}, align {:#?}",
            self.layout.size(),
            self.layout.align()
        )
    }
}

#[derive(Debug, Clone)]
pub struct LayoutError;

impl core::fmt::Display for LayoutError {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("invalid layout parameters")
    }
}

impl const From<CoreLayoutError> for LayoutError {
    #[inline]
    fn from(_: CoreLayoutError) -> Self {
        LayoutError
    }
}

/// Errors returned by the `RawVec`.
#[derive(Debug, Clone)]
pub enum RawVecError {
    /// Allocation error.
    Alloc(AllocError),
    /// Layout error.
    Layout(LayoutError),
}

impl core::fmt::Display for RawVecError {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            RawVecError::Alloc(err) => write!(f, "{}", err),
            RawVecError::Layout(err) => write!(f, "{}", err),
        }
    }
}

impl const From<AllocError> for RawVecError {
    #[inline]
    fn from(err: AllocError) -> Self {
        RawVecError::Alloc(err)
    }
}

impl const From<LayoutError> for RawVecError {
    #[inline]
    fn from(err: LayoutError) -> Self {
        RawVecError::Layout(err)
    }
}
