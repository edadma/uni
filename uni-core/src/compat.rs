// Compatibility module for std/no_std builds
// Provides common types that work in both environments

#[cfg(target_os = "none")]
extern crate alloc;

#[cfg(not(target_os = "none"))]
pub use std::{
    boxed::Box,
    collections::BTreeMap,
    fmt,
    format,
    rc::Rc,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(target_os = "none")]
pub use self::alloc::{
    boxed::Box,
    collections::BTreeMap,
    format,
    rc::Rc,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(target_os = "none")]
pub use core::fmt;
