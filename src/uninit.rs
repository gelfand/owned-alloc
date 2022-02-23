use crate::{AllocError, OwnedAlloc, RawVec};
use std::{
    alloc::{alloc, dealloc, Layout},
    marker::PhantomData,
    mem,
    ptr::NonNull,
};

pub struct UninitAlloc<T>
where
    T: ?Sized,
{
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T> Default for UninitAlloc<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> UninitAlloc<T> {
    #[inline]
    pub fn new() -> Self {
        Self::try_new().unwrap_or_else(|err| panic!("UninitAlloc::new: {}", err))
    }

    #[inline]
    pub fn try_new() -> Result<Self, AllocError> {
        let layout = Layout::new::<T>();
        let res = if layout.size() == 0 {
            Ok(NonNull::<T>::dangling())
        } else {
            NonNull::new(unsafe { alloc(layout) })
                .map(NonNull::cast::<T>)
                .ok_or(AllocError { layout })
        };
        res.map(|ptr| Self {
            ptr,
            _marker: PhantomData,
        })
    }

    #[inline]
    pub const fn init(self, value: T) -> OwnedAlloc<T> {
        let raw = self.into_raw();
        unsafe {
            raw.as_ptr().write(value);
            OwnedAlloc::from_raw(raw)
        }
    }
}

impl<T> UninitAlloc<T>
where
    T: ?Sized,
{
    #[inline]
    pub unsafe fn init_in_place<F>(self, init: F) -> OwnedAlloc<T>
    where
        F: FnOnce(&mut T),
    {
        let mut raw = self.into_raw();
        init(raw.as_mut());
        OwnedAlloc::from_raw(raw)
    }

    #[inline]
    pub const fn into_raw(self) -> NonNull<T> {
        let ptr = self.ptr;
        mem::forget(self);
        ptr
    }

    #[inline]
    pub const unsafe fn from_raw(ptr: NonNull<T>) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub const fn raw(&self) -> NonNull<T> {
        self.ptr
    }
}

impl<T> Drop for UninitAlloc<T>
where
    T: ?Sized,
{
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::for_value(self.ptr.as_ref());

            if layout.size() != 0 {
                dealloc(self.ptr.cast().as_ptr(), layout);
            }
        }
    }
}

impl<T> std::fmt::Debug for UninitAlloc<T>
where
    T: ?Sized,
{
    #[inline]
    fn fmt(&self, fmtr: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmtr, "{:?}", self.ptr)
    }
}

impl<T> const From<RawVec<T>> for UninitAlloc<[T]> {
    #[inline]
    fn from(alloc: RawVec<T>) -> Self {
        Self {
            ptr: alloc.into_raw_slice(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<T> const Send for UninitAlloc<T> where T: ?Sized + Send {}
unsafe impl<T> const Sync for UninitAlloc<T> where T: ?Sized + Sync {}

#[cfg(test)]
mod test {
    use super::UninitAlloc;

    #[test]
    fn into_from_raw() {
        let alloc = UninitAlloc::<usize>::new();
        let raw_borrowed = alloc.raw();
        let raw = alloc.into_raw();

        assert_eq!(raw, raw_borrowed);

        let alloc = unsafe { UninitAlloc::from_raw(raw) };
        assert_eq!(alloc.raw(), raw_borrowed);
    }
}
