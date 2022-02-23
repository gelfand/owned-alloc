use crate::{AllocError, LayoutError, RawVecError, UninitAlloc};
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout},
    marker::PhantomData,
    mem,
    ptr::NonNull,
};

pub struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T> RawVec<T> {
    /// Creates a new `RawVec` of capacity `0` and a dangling pointer. No
    /// allocation is performed.
    #[inline]
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
            _marker: PhantomData,
        }
    }

    /// Creates a new `RawVec` with a given capacity. In case of allocation
    /// error, the handler registered via stdlib is called. In case of overflow
    /// calculating the total size, the function panics.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        match Self::try_with_capacity(cap) {
            Ok(this) => this,
            Err(RawVecError::Alloc(err)) => handle_alloc_error(err.layout),
            Err(RawVecError::Layout(err)) => {
                panic!("Capacity overflows memory size: {}", err)
            }
        }
    }

    // Creates a new `RawVec` with a given capacity. In case of allocation
    /// error or overflow calculating the total size, `Err` is returned.
    #[inline]
    pub fn try_with_capacity(cap: usize) -> Result<Self, RawVecError> {
        let layout = Self::make_layout(cap)?;
        let res = if layout.size() == 0 {
            Ok(NonNull::dangling())
        } else {
            NonNull::new(unsafe { alloc(layout) })
                .map(NonNull::cast::<T>)
                .ok_or_else(|| AllocError { layout }.into())
        };

        res.map(|ptr| Self {
            ptr,
            cap,
            _marker: PhantomData,
        })
    }

    // Creates a `RawVec` from a plain old standard library `Vec`. Beware, only
    /// the pointer and the capacity are saved. The length is discarded. If you
    /// want to keep track of the length, you will have to store it for
    /// yourself. Note also that no element is dropped (ever) by the
    /// `RawVec`.
    ///
    /// # Safety
    /// This function is `unsafe` because there are no guarantees that `Vec` and
    /// `RawVec` allocate in the same way. They probably do in the Rust version
    /// you are using, but there are no future guarantees.
    #[inline]
    pub unsafe fn from_vec(mut vec: Vec<T>) -> Self {
        let this = Self {
            ptr: NonNull::new_unchecked(vec.as_mut_ptr()),
            cap: vec.capacity(),
            _marker: PhantomData,
        };
        mem::forget(vec);
        this
    }

    /// Recreate the `RawVec` from a raw non-null pointer and a capacity.
    ///
    /// # Safety
    /// This functions is `unsafe` because passing the wrong pointer leads to
    /// undefined behaviour. Passing wrong capacity also leads to undefined
    /// behaviour.
    #[inline]
    pub const unsafe fn from_raw_parts(ptr: NonNull<T>, cap: usize) -> Self {
        Self {
            ptr,
            cap,
            _marker: PhantomData,
        }
    }

    /// Recreate the `RawVec` from a raw non-null pointer to a slice with length
    /// equal to the `RawVec`'s capacity.
    ///
    /// # Safety
    /// This functions is `unsafe` because passing the wrong pointer leads to
    /// undefined behaviour, including passing a pointer with the wrong length.
    #[inline]
    pub const unsafe fn from_raw_slice(mut raw: NonNull<[T]>) -> Self {
        Self {
            ptr: NonNull::new_unchecked(raw.as_mut().as_mut_ptr()),
            cap: raw.as_ref().len(),
            _marker: PhantomData,
        }
    }

    /// The requested allocation capacity. It is guaranteed to be the capacity
    /// passed to the last capacity-modifier method. Those are
    /// `with_capacity`, `try_with_capacity` and `resize`. The methods `new`
    /// and `try_new` initialize the capacity to `0`.
    #[inline]
    pub const fn cap(&self) -> usize {
        self.cap
    }

    /// The raw non-null pointer to the first element.
    #[inline]
    pub const fn raw(&self) -> NonNull<T> {
        self.ptr
    }

    /// The raw non-null pointer to the slice with length equal to the
    /// `RawVec`'s capacity.
    #[inline]
    pub const fn raw_slice(&self) -> NonNull<[T]> {
        unsafe { NonNull::from(self.as_slice()) }
    }

    /// "Forgets" dropping the allocation and returns a raw non-null pointer to
    /// the slice with length equal to the `RawVec`'s capacity.
    #[inline]
    pub const fn into_raw_slice(self) -> NonNull<[T]> {
        let ptr = self.raw_slice();
        mem::forget(self);
        ptr
    }

    /// Encodes the `RawVec` as an immutable reference to a slice with length
    /// equal to the capacity.
    ///
    /// # Safety
    /// This function is `unsafe` because if the index of an uninitialized
    /// element is accessed incorrectly, undefined behavior occurs.
    #[inline]
    pub const unsafe fn as_slice(&self) -> &[T] {
        std::slice::from_raw_parts(self.ptr.as_ptr(), self.cap())
    }

    /// Encodes the `RawVec` as an mutable reference to a slice with length
    /// equal to the capacity.
    ///
    /// # Safety
    /// This function is `unsafe` because if the index of an uninitialized
    /// element is accessed incorrectly, undefined behavior occurs.
    #[inline]
    pub const unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.cap())
    }

    /// Creates a plain old standard library `Vec` from the `RawVec` and a given
    /// length.
    ///
    /// # Safety
    /// This function is `unsafe` because there are no guarantees that `Vec` and
    /// `RawVec` allocate in the same way. They probably do in the Rust version
    /// you are using, but there are no future guarantees. Also, the length
    /// argument must be passed correctly, since the elements until the given
    /// length will be considered correctly, but the `RawVec` initialize no
    /// element.
    #[inline]
    pub unsafe fn into_vec(self, len: usize) -> Vec<T> {
        let vec = Vec::from_raw_parts(self.ptr.as_ptr(), len, self.cap);
        mem::forget(self);
        vec
    }

    /// Resizes the `RawVec` with a given capacity. In case of allocation
    /// error, the handler registered via stdlib is called. In case of overflow
    /// calculating the total size, the function panics.
    #[inline]
    pub fn resize(&mut self, new_cap: usize) {
        match self.try_resize(new_cap) {
            Err(RawVecError::Alloc(err)) => handle_alloc_error(err.layout),
            Err(RawVecError::Layout(err)) => {
                panic!("Capacity overflows memory size: {}", err)
            }

            Ok(_) => (),
        }
    }

    /// Resizes the `RawVec` with a given capacity. In case of allocation
    /// error or overflow calculating the total size, `Err` is returned. In case
    /// of failure, the original allocation is untouched.
    #[inline]
    pub fn try_resize(&mut self, new_cap: usize) -> Result<(), RawVecError> {
        let layout = Self::make_layout(new_cap)?;

        let res = if layout.size() == 0 {
            self.free();
            Ok(NonNull::dangling())
        } else {
            let old = Self::make_layout(self.cap).unwrap();
            NonNull::new(unsafe { realloc(self.ptr.cast().as_ptr(), old, layout.size()) })
                .map(NonNull::cast::<T>)
                .ok_or_else(|| AllocError { layout }.into())
        };
        res.map(|ptr| {
            self.ptr = ptr;
            self.cap = new_cap;
        })
    }

    #[inline]
    fn free(&self) {
        if self.cap != 0 && mem::size_of::<T>() != 0 {
            let layout = Self::make_layout(self.cap).unwrap();
            unsafe {
                dealloc(self.ptr.cast().as_ptr(), layout);
            }
        }
    }

    #[inline]
    const fn make_layout(cap: usize) -> Result<Layout, LayoutError> {
        let total_size = mem::size_of::<T>().checked_mul(cap).ok_or(LayoutError)?;
        match Layout::from_size_align(total_size, mem::align_of::<T>()) {
            Ok(v) => Ok(v),
            Err(err) => Err(err.into()),
        }
    }
}

impl<T> std::fmt::Debug for RawVec<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RawVec {{ pointer {:?}, cap: {} }}", self.ptr, self.cap)
    }
}

impl<T> Drop for RawVec<T> {
    #[inline]
    fn drop(&mut self) {
        self.free();
    }
}

impl<T> const From<UninitAlloc<T>> for RawVec<T> {
    #[inline]
    fn from(alloc: UninitAlloc<T>) -> Self {
        Self {
            ptr: alloc.into_raw(),
            cap: 1,
            _marker: PhantomData,
        }
    }
}

unsafe impl<T> const Send for RawVec<T> where T: Send {}
unsafe impl<T> const Sync for RawVec<T> where T: Sync {}

#[cfg(test)]
mod test {
    use super::RawVec;

    #[test]
    fn cap_is_the_one_passed() {
        let mut alloc = RawVec::<usize>::with_capacity(20);
        assert_eq!(alloc.cap(), 20);

        alloc.resize(50);
        assert_eq!(alloc.cap(), 50);

        alloc.resize(5);
        assert_eq!(alloc.cap(), 5);
    }

    #[test]
    fn from_into_std_vec() {
        let vec = unsafe { RawVec::<u128>::with_capacity(465).into_vec(0) };
        assert_eq!(vec.capacity(), 465);
        let raw = unsafe { RawVec::from_vec(vec) };
        assert_eq!(raw.cap(), 465);
    }
}
