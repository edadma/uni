// RUST CONCEPT: Module organization
// This file declares all the primitive modules and re-exports their functions

// Basic arithmetic operations
pub mod plus;
pub mod minus;
pub mod multiply;
pub mod divide;
pub mod modulo;
pub mod equals;

// Comparison operations
pub mod less_than;
pub mod greater_than;
pub mod less_equal;
pub mod greater_equal;
pub mod not_equal;

// Basic math functions
pub mod abs;
pub mod min;
pub mod max;
pub mod sqrt;

// Advanced math functions
pub mod pow;
pub mod floor;
pub mod ceil;
pub mod round;

// Trigonometric functions
pub mod sin;
pub mod cos;
pub mod tan;

// Logarithmic functions
pub mod log;
pub mod exp;

// Bitwise operations
pub mod bit_and;
pub mod bit_or;
pub mod bit_xor;
pub mod bit_not;

// Shift operations
pub mod shl;
pub mod shr;

// Stack operations
pub mod drop;
// exec is now handled specially in the evaluator, not as a primitive
pub mod roll;
pub mod pick;
pub mod return_stack;

// List operations
pub mod cons;
pub mod list;
pub mod head;
pub mod tail;

// Meta operations
pub mod def;
pub mod val;

// Control flow - if is now handled specially in the evaluator

// I/O operations
pub mod print;

// Predicate operations
pub mod truthy;
pub mod null;

// Re-export all builtin functions for easy access

// Basic arithmetic
pub use plus::add_builtin;
pub use minus::sub_builtin;
pub use multiply::mul_builtin;
pub use divide::div_builtin;
pub use modulo::mod_builtin;
pub use equals::eq_builtin;

// Comparison operations
pub use less_than::less_than_builtin;
pub use greater_than::greater_than_builtin;
pub use less_equal::less_equal_builtin;
pub use greater_equal::greater_equal_builtin;
pub use not_equal::not_equal_builtin;

// Basic math functions
pub use abs::abs_builtin;
pub use min::min_builtin;
pub use max::max_builtin;
pub use sqrt::sqrt_builtin;

// Advanced math functions
pub use pow::pow_builtin;
pub use floor::floor_builtin;
pub use ceil::ceil_builtin;
pub use round::round_builtin;

// Trigonometric functions
pub use sin::sin_builtin;
pub use cos::cos_builtin;
pub use tan::tan_builtin;

// Logarithmic functions
pub use log::log_builtin;
pub use exp::exp_builtin;

// Bitwise operations
pub use bit_and::bit_and_builtin;
pub use bit_or::bit_or_builtin;
pub use bit_xor::bit_xor_builtin;
pub use bit_not::bit_not_builtin;

// Shift operations
pub use shl::shl_builtin;
pub use shr::shr_builtin;

// Stack operations
pub use drop::drop_builtin;
// exec is now handled specially in the evaluator
pub use roll::roll_builtin;
pub use pick::pick_builtin;

// Return stack operations
pub use return_stack::to_r_builtin;
pub use return_stack::from_r_builtin;
pub use return_stack::r_fetch_builtin;

// List operations
pub use cons::cons_builtin;
pub use list::list_builtin;
pub use head::head_builtin;
pub use tail::tail_builtin;

// Meta operations
pub use def::def_builtin;
pub use val::val_builtin;

// Control flow - if is now handled specially in the evaluator

// I/O operations
pub use print::print_builtin;

// Predicate operations
pub use truthy::truthy_predicate_builtin;
pub use null::null_predicate_builtin;

// RUST CONCEPT: Module-level documentation
// This explains the organization strategy for the primitives