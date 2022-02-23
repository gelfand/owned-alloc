#![allow(clippy::missing_safety_doc)]
#![feature(alloc_layout_extra)]
#![feature(allocator_api)]
#![feature(const_try)]
#![feature(const_heap)]
#![feature(const_ptr_as_ref)]
#![feature(const_fmt_arguments_new)]
#![feature(const_format_args)]
#![feature(const_option_ext)]
#![feature(const_nonnull_new)]
#![feature(rustc_allow_const_fn_unstable)]
#![feature(const_maybe_uninit_write)]
#![feature(array_chunks)]
#![feature(array_methods)]
#![feature(array_windows)]
#![feature(async_iterator)]
#![feature(coerce_unsized)]
#![cfg_attr(not(no_global_oom_handling), feature(const_alloc_error))]
#![feature(const_box)]
#![feature(box_into_inner)]
#![cfg_attr(not(no_global_oom_handling), feature(const_btree_new))]
#![feature(const_cow_is_borrowed)]
#![feature(const_convert)]
#![feature(const_size_of_val)]
#![feature(const_align_of_val)]
#![feature(const_ptr_read)]
#![feature(const_maybe_uninit_as_mut_ptr)]
#![feature(const_refs_to_cell)]
#![feature(const_eval_select)]
#![feature(const_pin)]
#![feature(dispatch_from_dyn)]
#![feature(exact_size_is_empty)]
#![feature(extend_one)]
#![feature(fn_traits)]
#![feature(inplace_iteration)]
#![feature(iter_advance_by)]
#![feature(layout_for_ptr)]
#![feature(maybe_uninit_slice)]
#![cfg_attr(test, feature(new_uninit))]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(pattern)]
#![feature(receiver_trait)]
#![feature(set_ptr_value)]
#![feature(slice_group_by)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
#![feature(slice_range)]
#![feature(trusted_len)]
#![feature(trusted_random_access)]
#![feature(try_trait_v2)]
#![feature(unsize)]
//
// // Language features:
#![feature(allocator_internals)]
#![feature(associated_type_bounds)]
#![feature(box_syntax)]
#![feature(cfg_sanitize)]
#![cfg_attr(bootstrap, feature(cfg_target_has_atomic))]
#![feature(const_deref)]
#![feature(const_fn_trait_bound)]
#![feature(const_mut_refs)]
#![feature(const_ptr_write)]
#![feature(const_precise_live_drops)]
#![feature(const_trait_impl)]
#![feature(dropck_eyepatch)]
#![feature(exclusive_range_pattern)]
#![feature(fundamental)]
#![cfg_attr(not(test), feature(generator_trait))]
#![feature(lang_items)]
#![feature(min_specialization)]
#![feature(negative_impls)]
#![feature(never_type)]
#![feature(nll)] // Not necessary, but here to test the `nll` feature.
#![cfg_attr(test, feature(test))]
#![feature(unboxed_closures)]
#![feature(unsized_fn_params)]
#![feature(c_unwind)]

pub mod owned;
pub mod uninit;
pub use owned::*;
pub use uninit::*;
