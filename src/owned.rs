use crate::{AllocError, UninitAlloc};
use std::{
    alloc::{dealloc, Global, Layout},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub struct OwnedAlloc<T>
where
    T: ?Sized,
{
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T> OwnedAlloc<T> {
    /// Creates an allocation and initializes it to the passed argument. In case
    /// of allocation error, the handler registered via stdlib is called.
    #[inline]
    pub fn new(value: T) -> Self {
        UninitAlloc::new().init(value)
    }

    #[inline]
    pub fn try_new(value: T) -> Result<Self, AllocError> {
        UninitAlloc::try_new().map(|alloc| alloc.init(value))
    }

    #[inline]
    pub const fn move_inner(self) -> (T, UninitAlloc<T>) {
        let val = unsafe { self.ptr.as_ptr().read() };
        let alloc = unsafe { UninitAlloc::from_raw(self.ptr) };
        mem::forget(self);
        (val, alloc)
    }
}

impl<T> OwnedAlloc<T>
where
    T: ?Sized,
{
    #[inline]
    pub const unsafe fn from_raw(ptr: NonNull<T>) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }
    #[inline]
    pub const unsafe fn from_box(boxed: Box<T>) -> Self {
        Self::from_raw(NonNull::new_unchecked(
            Box::into_raw_with_allocator(boxed).0,
        ))
    }
    #[inline]
    pub const fn raw(&self) -> NonNull<T> {
        self.ptr
    }
    #[inline]
    pub const fn into_raw(self) -> NonNull<T> {
        let ptr = self.ptr;
        mem::forget(self);
        ptr
    }
    #[inline]
    pub const unsafe fn into_box(self) -> Box<T> {
        Box::from_raw_in(self.into_raw().as_ptr(), Global)
    }

    #[inline]
    pub fn drop_in_place(self) -> UninitAlloc<T> {
        unsafe {
            self.ptr.as_ptr().drop_in_place();
            UninitAlloc::from_raw(self.into_raw())
        }
    }
}

impl<T> Drop for OwnedAlloc<T>
where
    T: ?Sized,
{
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::for_value(self.ptr.as_ref());
            self.ptr.as_ptr().drop_in_place();
            if layout.size() != 0 {
                dealloc(self.ptr.cast().as_ptr(), layout);
            }
        }
    }
}

impl<T> const Deref for OwnedAlloc<T>
where
    T: ?Sized,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> const DerefMut for OwnedAlloc<T>
where
    T: ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> std::fmt::Debug for OwnedAlloc<T>
where
    T: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "OwnedAlloc({:?})", self.ptr)
    }
}

impl<T> Clone for OwnedAlloc<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::new((**self).clone())
    }
}

impl<T> From<T> for OwnedAlloc<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

unsafe impl<T> const Send for OwnedAlloc<T> where T: ?Sized + Send {}
unsafe impl<T> const Sync for OwnedAlloc<T> where T: ?Sized + Sync {}

#[cfg(test)]
mod test {
    use super::OwnedAlloc;
    #[test]
    fn inner_eq() {
        let mut alloc = OwnedAlloc::new(20);

        assert_eq!(*alloc, 20);

        *alloc = 30;

        assert_eq!(*alloc, 30);
    }
    #[test]
    fn move_inner_eq() {
        let alloc = OwnedAlloc::new(20);

        assert_eq!(alloc.move_inner().0, 20);
    }
    #[test]
    fn from_into_std_box() {
        let boxed = unsafe { OwnedAlloc::new([5u128; 32]).into_box() };
        assert_eq!(*boxed, [5; 32]);
        let raw = unsafe { OwnedAlloc::from_box(boxed) };
        assert_eq!(*raw, [5; 32]);
    }
}
