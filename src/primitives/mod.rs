// RUST CONCEPT: Module organization
// This file declares all the primitive modules and re-exports their functions

// Arithmetic operations
pub mod plus;
pub mod minus;
pub mod multiply;
pub mod divide;
pub mod modulo;
pub mod equals;

// Stack operations
pub mod drop;
pub mod eval;

// List operations
pub mod cons;
pub mod list;
pub mod head;

// Stack operations
pub mod roll;
pub mod pick;

// List operations
pub mod tail;

// Meta operations
pub mod def;
pub mod val;

// Control flow
pub mod if_primitive;

// I/O operations
pub mod print;

// Predicate operations
pub mod truthy;
pub mod null;

// Re-export all builtin functions for easy access
pub use plus::add_builtin;
pub use minus::sub_builtin;
pub use multiply::mul_builtin;
pub use divide::div_builtin;
pub use modulo::mod_builtin;
pub use equals::eq_builtin;
pub use drop::drop_builtin;
pub use eval::eval_builtin;
pub use cons::cons_builtin;
pub use list::list_builtin;
pub use head::head_builtin;
pub use roll::roll_builtin;
pub use pick::pick_builtin;
pub use tail::tail_builtin;
pub use def::def_builtin;
pub use val::val_builtin;
pub use if_primitive::if_builtin;
pub use print::print_builtin;
pub use truthy::truthy_predicate_builtin;
pub use null::null_predicate_builtin;

// RUST CONCEPT: Module-level documentation
// This explains the organization strategy for the primitives