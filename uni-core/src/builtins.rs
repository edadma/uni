use crate::interpreter::{AsyncInterpreter, DictEntry};
use crate::value::{RuntimeError, Value};
use crate::compat::Box;
use core::future::Future;
use core::pin::Pin;

// Note: HashMap not needed here - dictionary is on AsyncInterpreter

// ASYNC CONCEPT: Helper type for async builtins
type AsyncBuiltinFn = fn(&mut AsyncInterpreter)
    -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>>;

// Sync helper macro for simple builtins
macro_rules! sync_builtin {
    ($func:expr) => {
        |interp: &mut AsyncInterpreter| -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>> {
            Box::pin(async move { $func(interp) })
        }
    };
}

pub fn register_async_builtins(interp: &mut AsyncInterpreter) {
    // Helper to add async builtin
    let add_builtin = |interp: &mut AsyncInterpreter, name: &str, func: AsyncBuiltinFn| {
        let atom = interp.intern_atom(name);
        interp.dictionary.insert(
            atom,
            DictEntry {
                value: Value::AsyncBuiltin(func),
                is_executable: true,
                doc: None,
            },
        );
    };

    // Async I/O primitives
    add_builtin(interp, ".", crate::primitives::print::print_builtin);
    add_builtin(interp, "cr", crate::primitives::cr::cr_builtin);
    add_builtin(interp, "words", crate::primitives::words::words_builtin);

    // Sync stack primitives (wrapped in async)
    add_builtin(interp, "dup", sync_builtin!(crate::primitives::stack::dup_impl));
    add_builtin(interp, "drop", sync_builtin!(crate::primitives::stack::drop_impl));
    add_builtin(interp, "swap", sync_builtin!(crate::primitives::stack::swap_impl));
    add_builtin(interp, "over", sync_builtin!(crate::primitives::stack::over_impl));
    add_builtin(interp, "rot", sync_builtin!(crate::primitives::stack::rot_impl));

    // Sync arithmetic primitives (wrapped in async)
    add_builtin(interp, "+", sync_builtin!(crate::primitives::plus::add_impl));
    add_builtin(interp, "-", sync_builtin!(crate::primitives::minus::sub_impl));

    // Sync comparison primitives (wrapped in async)
    add_builtin(interp, "=", sync_builtin!(crate::primitives::equals::equals_impl));
    add_builtin(interp, "<", sync_builtin!(crate::primitives::less_than::less_than_impl));
    add_builtin(interp, ">", sync_builtin!(crate::primitives::greater_than::greater_than_impl));
    add_builtin(interp, "<=", sync_builtin!(crate::primitives::less_equal::less_equal_impl));
    add_builtin(interp, ">=", sync_builtin!(crate::primitives::greater_equal::greater_equal_impl));

    // Sync definition primitives (wrapped in async)
    add_builtin(interp, "def", sync_builtin!(crate::primitives::def::def_impl));
    add_builtin(interp, "val", sync_builtin!(crate::primitives::val::val_impl));

    // Sync list primitives (wrapped in async)
    add_builtin(interp, "cons", sync_builtin!(crate::primitives::cons::cons_impl));
    add_builtin(interp, "car", sync_builtin!(crate::primitives::head::car_impl));
    add_builtin(interp, "cdr", sync_builtin!(crate::primitives::tail::cdr_impl));
}
