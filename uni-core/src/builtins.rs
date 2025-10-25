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
    // Note: swap, dup, over, rot are defined in the prelude using pick and roll
    add_builtin(interp, "drop", sync_builtin!(crate::primitives::stack::drop_impl));
    add_builtin(interp, "pick", sync_builtin!(crate::primitives::pick::pick_impl));
    add_builtin(interp, "roll", sync_builtin!(crate::primitives::roll::roll_impl));

    // Sync arithmetic primitives (wrapped in async)
    add_builtin(interp, "+", sync_builtin!(crate::primitives::plus::add_impl));
    add_builtin(interp, "-", sync_builtin!(crate::primitives::minus::sub_impl));
    add_builtin(interp, "*", sync_builtin!(crate::primitives::multiply::mul_impl));
    add_builtin(interp, "/", sync_builtin!(crate::primitives::divide::div_impl));
    add_builtin(interp, "mod", sync_builtin!(crate::primitives::modulo::mod_impl));
    add_builtin(interp, "//", sync_builtin!(crate::primitives::floor_div::floor_div_impl));
    add_builtin(interp, "div", sync_builtin!(crate::primitives::trunc_div::trunc_div_impl));
    add_builtin(interp, "abs", sync_builtin!(crate::primitives::abs::abs_impl));
    add_builtin(interp, "min", sync_builtin!(crate::primitives::min::min_impl));
    add_builtin(interp, "max", sync_builtin!(crate::primitives::max::max_impl));

    // Sync comparison primitives (wrapped in async)
    add_builtin(interp, "=", sync_builtin!(crate::primitives::equals::equals_impl));
    add_builtin(interp, "!=", sync_builtin!(crate::primitives::not_equal::not_equal_impl));
    add_builtin(interp, "<", sync_builtin!(crate::primitives::less_than::less_than_impl));
    add_builtin(interp, ">", sync_builtin!(crate::primitives::greater_than::greater_than_impl));
    add_builtin(interp, "<=", sync_builtin!(crate::primitives::less_equal::less_equal_impl));
    add_builtin(interp, ">=", sync_builtin!(crate::primitives::greater_equal::greater_equal_impl));

    // Sync definition primitives (wrapped in async)
    add_builtin(interp, "def", sync_builtin!(crate::primitives::def::def_impl));
    add_builtin(interp, "val", sync_builtin!(crate::primitives::val::val_impl));
    add_builtin(interp, "doc", sync_builtin!(crate::primitives::doc::doc_impl));

    // Sync list primitives (wrapped in async)
    add_builtin(interp, "cons", sync_builtin!(crate::primitives::cons::cons_impl));
    add_builtin(interp, "car", sync_builtin!(crate::primitives::head::car_impl));
    add_builtin(interp, "cdr", sync_builtin!(crate::primitives::tail::cdr_impl));

    // Date/time primitives (wrapped in async)
    add_builtin(interp, "now", sync_builtin!(crate::primitives::now::now_impl));

    // Advanced math primitives (feature-gated, wrapped in async)
    #[cfg(feature = "advanced_math")]
    {
        add_builtin(interp, "sqrt", sync_builtin!(crate::primitives::sqrt::sqrt_impl));
        add_builtin(interp, "pow", sync_builtin!(crate::primitives::pow::pow_impl));
        add_builtin(interp, "floor", sync_builtin!(crate::primitives::floor::floor_impl));
        add_builtin(interp, "ceil", sync_builtin!(crate::primitives::ceil::ceil_impl));
        add_builtin(interp, "round", sync_builtin!(crate::primitives::round::round_impl));
        add_builtin(interp, "sin", sync_builtin!(crate::primitives::sin::sin_impl));
        add_builtin(interp, "cos", sync_builtin!(crate::primitives::cos::cos_impl));
        add_builtin(interp, "tan", sync_builtin!(crate::primitives::tan::tan_impl));
        add_builtin(interp, "log", sync_builtin!(crate::primitives::log::log_impl));
        add_builtin(interp, "exp", sync_builtin!(crate::primitives::exp::exp_impl));
    }

    // Record primitives (wrapped in async)
    add_builtin(interp, "make-record-type", sync_builtin!(crate::primitives::record::make_record_type_impl));
    add_builtin(interp, "construct-record", sync_builtin!(crate::primitives::record::construct_record_impl));
    add_builtin(interp, "is-record-type?", sync_builtin!(crate::primitives::record::is_record_type_impl));
    add_builtin(interp, "get-record-field", sync_builtin!(crate::primitives::record::get_record_field_impl));
    add_builtin(interp, "set-record-field!", sync_builtin!(crate::primitives::record::set_record_field_impl));
    add_builtin(interp, "record-type-of", sync_builtin!(crate::primitives::record::record_type_of_impl));

    // Create the date record type for use by the 'now' primitive
    // Field names: year month day hour minute second offset
    create_date_record_type(interp);
}

// Helper function to create the date record type used by 'now'
fn create_date_record_type(interp: &mut AsyncInterpreter) {
    use crate::compat::Rc;
    use crate::primitives::record::make_record_type_impl;

    // Build field names list: (year month day hour minute second offset)
    let field_names = vec!["year", "month", "day", "hour", "minute", "second", "offset"];
    let mut field_list = Value::Nil;
    for name in field_names.iter().rev() {
        let name_atom = interp.intern_atom(name);
        field_list = Value::Pair(
            Rc::new(Value::Atom(name_atom)),
            Rc::new(field_list)
        );
    }

    // Push arguments for make-record-type: field_list type_name
    interp.push(field_list);
    let date_atom = interp.intern_atom("date");
    interp.push(Value::String(date_atom));

    // Create the record type
    if let Err(e) = make_record_type_impl(interp) {
        // This should not fail during initialization
        panic!("Failed to create date record type: {:?}", e);
    }

    // Pop the record type that was returned
    interp.pop().expect("Expected record type on stack");
}
