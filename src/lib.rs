#![no_std]
#![allow(clippy::missing_safety_doc)]
#![feature(allocator_api)]
#![feature(const_option_ext)]
#![feature(const_ptr_offset)]
#![feature(const_convert)]
#![feature(const_try)]
#![feature(const_mut_refs)]
#![feature(const_slice_from_raw_parts)]
#![feature(alloc_layout_extra)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(const_ptr_as_ref)]
#![feature(const_maybe_uninit_write)]
#![feature(const_box)]
#![feature(const_ptr_write)]
#![feature(box_into_inner)]
#![feature(const_ptr_read)]
#![feature(const_maybe_uninit_as_mut_ptr)]
#![feature(const_eval_select)]
#![feature(const_precise_live_drops)]
#![feature(const_trait_impl)]
#![feature(const_fn_trait_bound)]
#![feature(const_option)]
#![feature(unboxed_closures)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]

pub mod cache;
pub mod error;
pub mod maybe_uninit;
pub mod owned;
pub mod raw_vec;
pub mod uninit;
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};

use alloc::alloc::{alloc_zeroed, dealloc};
pub use cache::*;
pub use error::*;
pub use maybe_uninit::*;
pub use owned::*;
pub use raw_vec::*;
pub use uninit::*;

extern crate alloc;
pub struct Allocator {}

static mut ALLOCATOR: Allocator = Allocator {};

///
///
/// #[global_allocator]
/// static ALLOCATOR: SimpleAllocator = SimpleAllocator {
///     arena: UnsafeCell::new([0x55; ARENA_SIZE]),

unsafe impl alloc::alloc::GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let (size, align) = (layout.size(), layout.align());
        let ptr = alloc_zeroed(layout);
        if !ptr.is_null() {
            let offset = ptr.align_offset(align);
            if offset == 0 {
                ptr
            } else {
                let new_ptr = ptr.add(offset);
                // SAFETY: the region from `new_ptr` of size `size` is guaranteed to be valid for writes.
                core::ptr::write_bytes(new_ptr, 0, size);
                new_ptr
            }
        } else {
            ptr
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = (layout.size(), layout.align());
        // SAFETY: the region from `ptr` of size `size` is guaranteed to be valid for writes.
        core::ptr::write_bytes(ptr, 0, size);
        // SAFETY: the region from `ptr` of size `size` is guaranteed to be valid for writes.
        dealloc(ptr, layout)
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        let size = layout.size();
        // SAFETY: the safety contract for `alloc` must be upheld by the caller.
        let ptr = self.alloc(layout);
        if !ptr.is_null() {
            // SAFETY: as allocation succeeded, the region from `ptr`
            // of size `size` is guaranteed to be valid for writes.
            core::ptr::write_bytes(ptr, 0, size);
        }
        ptr
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        // SAFETY: the caller must ensure that the `new_size` does not overflow.
        // `layout.align()` comes from a `Layout` and is thus guaranteed to be valid.
        let new_layout = core::alloc::Layout::from_size_align_unchecked(new_size, layout.align());
        // SAFETY: the caller must ensure that `new_layout` is greater than zero.
        let new_ptr = self.alloc(new_layout);
        if !new_ptr.is_null() {
            // SAFETY: the previously allocated block cannot overlap the newly allocated block.
            // The safety contract for `dealloc` must be upheld by the caller.
            core::ptr::copy_nonoverlapping(ptr, new_ptr, core::cmp::min(layout.size(), new_size));
            self.dealloc(ptr, layout);
        }
        new_ptr
    }
}

unsafe impl core::alloc::Allocator for Allocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        self.alloc_impl(layout, false)
    }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() == 0 {
            GlobalAlloc::dealloc(&ALLOCATOR, ptr.as_ptr(), layout);
        }
    }
    fn allocate_zeroed(
        &self,
        layout: Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        self.alloc_impl(layout, true)
    }
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {}
    }

    fn alloc_impl(
        &self,
        layout: Layout,
        zeroed: bool,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        match layout.size() {
            0 => Ok(NonNull::slice_from_raw_parts(
                unsafe { NonNull::new_unchecked(layout.align() as *mut u8) },
                0,
            )),
            // SAFETY: `layout` is non-zero in size,
            size => unsafe {
                let raw_ptr = if zeroed {
                    GlobalAlloc::alloc_zeroed(self, layout)
                } else {
                    GlobalAlloc::alloc(self, layout)
                };
                let ptr = match NonNull::new(raw_ptr) {
                    Some(ptr) => ptr,
                    None => return Err(core::alloc::AllocError),
                };
                Ok(NonNull::slice_from_raw_parts(ptr, size))
            },
        }
    }

    unsafe fn deallocate<T>(&self, ptr: *mut T, layout: Layout) {
        GlobalAlloc::dealloc(self, ptr as *mut u8, layout)
    }

    unsafe fn reallocate<T>(
        &self,
        ptr: *mut T,
        layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut T {
        // SAFETY: the caller must ensure that the `new_size` does not overflow.
        // `layout.align()` comes from a `Layout` and is thus guaranteed to be valid.
        let new_layout = core::alloc::Layout::from_size_align_unchecked(new_size, layout.align());
        // SAFETY: the caller must ensure that `new_layout` is greater than zero.
        let new_ptr = self.alloc(new_layout) as *mut T;
        if !new_ptr.is_null() {
            // SAFETY: the previously allocated block cannot overlap the newly allocated block.
            // The safety contract for `dealloc` must be upheld by the caller.
            core::ptr::copy_nonoverlapping(ptr, new_ptr, core::cmp::min(layout.size(), new_size));
            self.deallocate(ptr, layout);
        }
        new_ptr
    }
}

impl Default for Allocator {
    fn default() -> Self {
        Self::new()
    }
}
