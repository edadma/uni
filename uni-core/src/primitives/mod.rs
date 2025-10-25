// Primitives module - organized into separate files for maintainability

// Async I/O primitives
pub mod print;
pub mod cr;
pub mod words;

// Stack manipulation
pub mod stack;
pub mod pick;
pub mod roll;

// Arithmetic
pub mod plus;
pub mod minus;
pub mod multiply;
pub mod divide;
pub mod modulo;
pub mod floor_div;
pub mod trunc_div;
pub mod abs;
pub mod min;
pub mod max;

// Advanced math (feature-gated)
#[cfg(feature = "advanced_math")]
pub mod sqrt;
#[cfg(feature = "advanced_math")]
pub mod pow;
#[cfg(feature = "advanced_math")]
pub mod floor;
#[cfg(feature = "advanced_math")]
pub mod ceil;
#[cfg(feature = "advanced_math")]
pub mod round;
#[cfg(feature = "advanced_math")]
pub mod sin;
#[cfg(feature = "advanced_math")]
pub mod cos;
#[cfg(feature = "advanced_math")]
pub mod tan;
#[cfg(feature = "advanced_math")]
pub mod log;
#[cfg(feature = "advanced_math")]
pub mod exp;

// Comparisons
pub mod equals;
pub mod not_equal;
pub mod less_than;
pub mod greater_than;
pub mod less_equal;
pub mod greater_equal;

// Definitions
pub mod def;
pub mod val;
pub mod doc;

// Lists
pub mod cons;
pub mod head;
pub mod tail;

// Date/time
pub mod now;

// Records
pub mod record;

// Numeric type promotion
pub mod numeric_promotion;
