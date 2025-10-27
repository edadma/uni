// Compatibility module for std/no_std builds
// Provides common types that work in both environments

#[cfg(target_os = "none")]
extern crate alloc;

#[cfg(not(target_os = "none"))]
pub use std::{
    boxed::Box,
    fmt,
    format,
    rc::Rc,
    sync::Arc,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(not(target_os = "none"))]
#[allow(unused_imports)]
pub use std::vec;

#[cfg(target_os = "none")]
pub use self::alloc::{
    boxed::Box,
    format,
    rc::Rc,
    sync::Arc,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(target_os = "none")]
#[allow(unused_imports)]
pub use self::alloc::vec;

#[cfg(target_os = "none")]
pub use core::fmt;
