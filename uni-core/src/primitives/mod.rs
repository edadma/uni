// RUST CONCEPT: Module organization
// This file declares all the primitive modules and re-exports their functions

// Basic arithmetic operations
pub mod divide;
pub mod equals;
pub mod floor_div;
pub mod minus;
pub mod modulo;
pub mod multiply;
pub mod plus;
pub mod trunc_div;

// Comparison operations
pub mod greater_equal;
pub mod greater_than;
pub mod less_equal;
pub mod less_than;
pub mod not_equal;

// Basic math functions
pub mod abs;
pub mod max;
pub mod min;
#[cfg(feature = "advanced_math")]
pub mod sqrt;

// Advanced math functions
#[cfg(feature = "advanced_math")]
pub mod ceil;
#[cfg(feature = "advanced_math")]
pub mod floor;
#[cfg(feature = "advanced_math")]
pub mod pow;
#[cfg(feature = "advanced_math")]
pub mod round;

// Trigonometric functions
#[cfg(feature = "advanced_math")]
pub mod cos;
#[cfg(feature = "advanced_math")]
pub mod sin;
#[cfg(feature = "advanced_math")]
pub mod tan;

// Logarithmic functions
#[cfg(feature = "advanced_math")]
pub mod exp;
#[cfg(feature = "advanced_math")]
pub mod log;

// Bitwise operations
pub mod bit_and;
pub mod bit_not;
pub mod bit_or;
pub mod bit_xor;

// Shift operations
pub mod shl;
pub mod shr;

// Stack operations
pub mod drop;
// exec is now handled specially in the evaluator, not as a primitive
pub mod pick;
pub mod return_stack;
pub mod roll;

// List operations
pub mod cons;
pub mod head;
pub mod list;
pub mod tail;

// Vector (array) operations
pub mod vector;

// Record operations
pub mod record;

// Time operations (platform-agnostic via TimeSource)
pub mod current_timestamp;
pub mod current_offset;

// Meta operations
pub mod def;
pub mod doc;
pub mod help;
pub mod val;

// Variable operations (Forth-style)
pub mod var;
pub mod fetch;  // @
pub mod store;  // !

// Local constants (lexically scoped)
pub mod lval;

// Control flow - if is now handled specially in the evaluator

// I/O operations
pub mod print;
pub mod words;
pub mod cr;
pub mod space;

// Stack management
pub mod clear;
pub mod stack;

// String operations
pub mod to_string;

// Predicate operations
pub mod null;
pub mod truthy;

// Type introspection
pub mod type_of;

// Numeric type promotion
pub mod numeric_promotion;

// I16 buffer operations (audio/DSP)
pub mod i16_buffer;
pub mod i16_ref;
pub mod i16_set;
pub mod i16_ops;
pub mod i16_dsp;

// Re-export all builtin functions for easy access

// Basic arithmetic
pub use divide::div_builtin;
pub use equals::eq_builtin;
pub use floor_div::floor_div_builtin;
pub use minus::sub_builtin;
pub use modulo::mod_builtin;
pub use multiply::mul_builtin;
pub use plus::add_builtin;
pub use trunc_div::trunc_div_builtin;

// Comparison operations
pub use greater_equal::greater_equal_builtin;
pub use greater_than::greater_than_builtin;
pub use less_equal::less_equal_builtin;
pub use less_than::less_than_builtin;
pub use not_equal::not_equal_builtin;

// Basic math functions
pub use abs::abs_builtin;
pub use max::max_builtin;
pub use min::min_builtin;
#[cfg(feature = "advanced_math")]
pub use sqrt::sqrt_builtin;

// Advanced math functions
#[cfg(feature = "advanced_math")]
pub use ceil::ceil_builtin;
#[cfg(feature = "advanced_math")]
pub use floor::floor_builtin;
#[cfg(feature = "advanced_math")]
pub use pow::pow_builtin;
#[cfg(feature = "advanced_math")]
pub use round::round_builtin;

// Trigonometric functions
#[cfg(feature = "advanced_math")]
pub use cos::cos_builtin;
#[cfg(feature = "advanced_math")]
pub use sin::sin_builtin;
#[cfg(feature = "advanced_math")]
pub use tan::tan_builtin;

// Logarithmic functions
#[cfg(feature = "advanced_math")]
pub use exp::exp_builtin;
#[cfg(feature = "advanced_math")]
pub use log::log_builtin;

// Bitwise operations
pub use bit_and::bit_and_builtin;
pub use bit_not::bit_not_builtin;
pub use bit_or::bit_or_builtin;
pub use bit_xor::bit_xor_builtin;

// Shift operations
pub use shl::shl_builtin;
pub use shr::shr_builtin;

// Stack operations
pub use drop::drop_builtin;
// exec is now handled specially in the evaluator
pub use pick::pick_builtin;
pub use roll::roll_builtin;

// Return stack operations
pub use return_stack::from_r_builtin;
pub use return_stack::r_fetch_builtin;
pub use return_stack::to_r_builtin;

// List operations
pub use cons::cons_builtin;
pub use head::head_builtin;
pub use list::list_builtin;
pub use tail::tail_builtin;

// Vector (array) operations
pub use vector::list_to_vector_builtin;
pub use vector::make_vector_builtin;
pub use vector::vector_builtin;
pub use vector::vector_length_builtin;
pub use vector::vector_ref_builtin;
pub use vector::vector_set_builtin;
pub use vector::vector_to_list_builtin;

// Record operations
pub use record::construct_record_builtin;
pub use record::get_record_field_builtin;
pub use record::is_record_type_builtin;
pub use record::make_record_type_builtin;
pub use record::record_type_of_builtin;
pub use record::set_record_field_builtin;

// Time operations
pub use current_timestamp::current_timestamp_builtin;
pub use current_offset::current_offset_builtin;

// Meta operations
pub use def::def_builtin;
pub use doc::doc_builtin;
pub use help::help_builtin;
pub use val::val_builtin;

// Variable operations (Forth-style)
pub use var::var_builtin;
pub use fetch::fetch_builtin;
pub use store::store_builtin;

// Local constants (lexically scoped)
pub use lval::lval_builtin;

// Control flow - if is now handled specially in the evaluator

// I/O operations
pub use print::print_builtin;
pub use words::words_builtin;
pub use cr::cr_builtin;
pub use space::space_builtin;

// Stack management
pub use clear::clear_builtin;
pub use stack::stack_builtin;

// String operations
pub use to_string::to_string_builtin;

// Predicate operations
pub use null::null_predicate_builtin;
pub use truthy::truthy_predicate_builtin;

// Type introspection
pub use type_of::type_of_builtin;

// I16 buffer operations
pub use i16_buffer::i16_buffer_builtin;
pub use i16_dsp::{i16_avg_builtin, i16_max_builtin, i16_min_builtin};
pub use i16_ops::{i16_length_builtin, i16_pop_builtin, i16_push_builtin};
pub use i16_ref::i16_ref_builtin;
pub use i16_set::i16_set_builtin;

// RUST CONCEPT: Module-level documentation
// This explains the organization strategy for the primitives
