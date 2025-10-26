// Primitives module - organized into separate files for maintainability

// Async I/O primitives
pub mod print;
pub mod emit;
pub mod words;
pub mod space;

// Async concurrency primitives
pub mod delay;
pub mod spawn;

// Utility primitives
pub mod help;
pub mod clear;

// Stack manipulation
pub mod stack;
pub mod pick;
pub mod roll;
pub mod return_stack;

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

// Variables
pub mod var;
pub mod fetch;
pub mod store;
pub mod lval;
pub mod lvar;

// Lists
pub mod cons;
pub mod head;
pub mod tail;
pub mod list;

// Vectors
pub mod vector;

// Date/time (now superseded by platform-specific primitives in hardware/)
// pub mod now;

// Records
pub mod record;

// Numeric type promotion
pub mod numeric_promotion;

// Bitwise operations
pub mod bit_and;
pub mod bit_or;
pub mod bit_xor;
pub mod bit_not;
pub mod shl;
pub mod shr;

// String/type operations
pub mod to_string;
pub mod truthy;
pub mod type_of;

// I32/F32 buffers (for audio/DSP/computation)
pub mod i32_buffer;
pub mod f32_buffer;
