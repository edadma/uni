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

// Predicate operations
pub mod truthy;

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
pub use truthy::truthy_predicate_builtin;

// RUST CONCEPT: Module-level documentation
// This explains the organization strategy for the primitives