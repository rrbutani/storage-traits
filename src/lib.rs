//! Some traits for various storage mediums.
//!
//! Useful for embedded applications.

#![forbid(
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused_allocation,
    unused_lifetimes,
    unused_comparisons,
    unused_parens,
    while_true
)]
#![deny(
    unused,
    bad_style,
    missing_debug_implementations,
    intra_doc_link_resolution_failure,
    // missing_docs, // TODO!
    unsafe_code,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    rust_2018_idioms
)]
#![doc(test(attr(deny(warnings))))]
#![doc(html_logo_url = "")] // TODO!

// Mark the crate as no_std if the `no_std` feature is enabled.
#![cfg_attr(feature = "no_std", no_std)]

macro_rules! using_std { ($($i:item)*) => ($(#[cfg(not(feature = "no_std"))]$i)*) }

#[allow(unused_extern_crates)]
extern crate core; // makes rls actually look into the standard library (hack)


mod bytes;
pub use bytes::*;

mod storage;
pub use storage::*;

mod extensions;
pub use extensions::*;

pub mod errors;
