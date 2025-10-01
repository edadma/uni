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
pub mod sqrt;

// Advanced math functions
pub mod ceil;
pub mod floor;
pub mod pow;
pub mod round;

// Trigonometric functions
pub mod cos;
pub mod sin;
pub mod tan;

// Logarithmic functions
pub mod exp;
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

// Meta operations
pub mod def;
pub mod doc;
pub mod help;
pub mod val;

// Control flow - if is now handled specially in the evaluator

// I/O operations
pub mod print;

// String operations
pub mod to_string;

// Predicate operations
pub mod null;
pub mod truthy;

// Type introspection
pub mod type_of;

// Numeric type promotion
pub mod numeric_promotion;

// Re-export all builtin functions for easy access

// Basic arithmetic
pub use divide::div_builtin;
pub use equals::eq_builtin;
pub use floor_div::floor_div_builtin;
pub use minus::sub_builtin;
pub use modulo::mod_builtin;
pub use multiply::mul_builtin;
pub use plus::add_builtin;

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
pub use sqrt::sqrt_builtin;

// Advanced math functions
pub use ceil::ceil_builtin;
pub use floor::floor_builtin;
pub use pow::pow_builtin;
pub use round::round_builtin;

// Trigonometric functions
pub use cos::cos_builtin;
pub use sin::sin_builtin;
pub use tan::tan_builtin;

// Logarithmic functions
pub use exp::exp_builtin;
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

// Meta operations
pub use def::def_builtin;
pub use doc::doc_builtin;
pub use help::help_builtin;
pub use val::val_builtin;

// Control flow - if is now handled specially in the evaluator

// I/O operations
pub use print::print_builtin;

// String operations
pub use to_string::to_string_builtin;

// Predicate operations
pub use null::null_predicate_builtin;
pub use truthy::truthy_predicate_builtin;

// Type introspection
pub use type_of::type_of_builtin;

// RUST CONCEPT: Module-level documentation
// This explains the organization strategy for the primitives
