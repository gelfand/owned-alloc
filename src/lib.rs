#![allow(clippy::missing_safety_doc)]
#![feature(allocator_api)]
#![feature(const_option_ext)]
#![feature(const_ptr_offset)]
#![feature(const_convert)]
#![feature(const_try)]
#![feature(const_mut_refs)]
#![feature(const_slice_from_raw_parts)]
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
#![feature(const_option)]

pub mod cache;
pub mod error;
pub mod maybe_uninit;
pub mod owned;
pub mod raw_vec;
pub mod uninit;
pub use cache::*;
pub use error::*;
pub use maybe_uninit::*;
pub use owned::*;
pub use raw_vec::*;
pub use uninit::*;
