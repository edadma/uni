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
    add_builtin(interp, "emit", crate::primitives::emit::emit_builtin);
    add_builtin(interp, "words", crate::primitives::words::words_builtin);
    add_builtin(interp, "space", crate::primitives::space::space_builtin);
    // Note: cr is now defined in the prelude as [10 emit]

    // Utility primitives
    add_builtin(interp, "help", crate::primitives::help::help_builtin);
    add_builtin(interp, "stack", crate::primitives::stack::stack_builtin);
    add_builtin(interp, "clear", sync_builtin!(crate::primitives::clear::clear_impl));

    // Sync stack primitives (wrapped in async)
    // Note: swap, dup, over, rot are defined in the prelude using pick and roll
    add_builtin(interp, "drop", sync_builtin!(crate::primitives::stack::drop_impl));
    add_builtin(interp, "pick", sync_builtin!(crate::primitives::pick::pick_impl));
    add_builtin(interp, "roll", sync_builtin!(crate::primitives::roll::roll_impl));

    // Return stack primitives
    add_builtin(interp, ">r", sync_builtin!(crate::primitives::return_stack::to_r_impl));
    add_builtin(interp, "r>", sync_builtin!(crate::primitives::return_stack::from_r_impl));
    add_builtin(interp, "r@", sync_builtin!(crate::primitives::return_stack::r_fetch_impl));

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

    // Variable primitives
    add_builtin(interp, "var", sync_builtin!(crate::primitives::var::var_impl));
    add_builtin(interp, "@", sync_builtin!(crate::primitives::fetch::fetch_impl));
    add_builtin(interp, "!", sync_builtin!(crate::primitives::store::store_impl));
    add_builtin(interp, "lval", sync_builtin!(crate::primitives::lval::lval_impl));
    add_builtin(interp, "lvar", sync_builtin!(crate::primitives::lvar::lvar_impl));

    // Sync list primitives (wrapped in async)
    add_builtin(interp, "cons", sync_builtin!(crate::primitives::cons::cons_impl));
    add_builtin(interp, "car", sync_builtin!(crate::primitives::head::car_impl));
    add_builtin(interp, "cdr", sync_builtin!(crate::primitives::tail::cdr_impl));
    add_builtin(interp, "list", sync_builtin!(crate::primitives::list::list_impl));

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

    // Vector primitives
    add_builtin(interp, "vector", sync_builtin!(crate::primitives::vector::vector_impl));
    add_builtin(interp, "make-vector", sync_builtin!(crate::primitives::vector::make_vector_impl));
    add_builtin(interp, "vector-length", sync_builtin!(crate::primitives::vector::vector_length_impl));
    add_builtin(interp, "vector-ref", sync_builtin!(crate::primitives::vector::vector_ref_impl));
    add_builtin(interp, "vector-set!", sync_builtin!(crate::primitives::vector::vector_set_impl));
    add_builtin(interp, "vector->list", sync_builtin!(crate::primitives::vector::vector_to_list_impl));
    add_builtin(interp, "list->vector", sync_builtin!(crate::primitives::vector::list_to_vector_impl));

    // Bitwise primitives
    add_builtin(interp, "&", sync_builtin!(crate::primitives::bit_and::bit_and_impl));
    add_builtin(interp, "|", sync_builtin!(crate::primitives::bit_or::bit_or_impl));
    add_builtin(interp, "^", sync_builtin!(crate::primitives::bit_xor::bit_xor_impl));
    add_builtin(interp, "~", sync_builtin!(crate::primitives::bit_not::bit_not_impl));
    add_builtin(interp, "<<", sync_builtin!(crate::primitives::shl::shl_impl));
    add_builtin(interp, ">>", sync_builtin!(crate::primitives::shr::shr_impl));

    // String/type primitives
    add_builtin(interp, "->string", sync_builtin!(crate::primitives::to_string::to_string_impl));
    add_builtin(interp, "truthy?", sync_builtin!(crate::primitives::truthy::truthy_impl));
    add_builtin(interp, "type-of", sync_builtin!(crate::primitives::type_of::type_of_impl));

    // I32 buffer primitives (for integer data and DSP)
    add_builtin(interp, "i32-buffer", sync_builtin!(crate::primitives::i32_buffer::i32_buffer_impl));
    add_builtin(interp, "i32@", sync_builtin!(crate::primitives::i32_buffer::i32_ref_impl));
    add_builtin(interp, "i32!", sync_builtin!(crate::primitives::i32_buffer::i32_set_impl));
    add_builtin(interp, "i32-length", sync_builtin!(crate::primitives::i32_buffer::i32_length_impl));
    add_builtin(interp, "i32-push!", sync_builtin!(crate::primitives::i32_buffer::i32_push_impl));
    add_builtin(interp, "i32-pop!", sync_builtin!(crate::primitives::i32_buffer::i32_pop_impl));
    add_builtin(interp, "i32-max", sync_builtin!(crate::primitives::i32_buffer::i32_max_impl));
    add_builtin(interp, "i32-min", sync_builtin!(crate::primitives::i32_buffer::i32_min_impl));
    add_builtin(interp, "i32-avg", sync_builtin!(crate::primitives::i32_buffer::i32_avg_impl));

    // F32 buffer primitives (for float data, audio/DSP, GPU compute)
    add_builtin(interp, "f32-buffer", sync_builtin!(crate::primitives::f32_buffer::f32_buffer_impl));
    add_builtin(interp, "f32@", sync_builtin!(crate::primitives::f32_buffer::f32_ref_impl));
    add_builtin(interp, "f32!", sync_builtin!(crate::primitives::f32_buffer::f32_set_impl));
    add_builtin(interp, "f32-length", sync_builtin!(crate::primitives::f32_buffer::f32_length_impl));
    add_builtin(interp, "f32-push!", sync_builtin!(crate::primitives::f32_buffer::f32_push_impl));
    add_builtin(interp, "f32-pop!", sync_builtin!(crate::primitives::f32_buffer::f32_pop_impl));
    add_builtin(interp, "f32-max", sync_builtin!(crate::primitives::f32_buffer::f32_max_impl));
    add_builtin(interp, "f32-min", sync_builtin!(crate::primitives::f32_buffer::f32_min_impl));
    add_builtin(interp, "f32-avg", sync_builtin!(crate::primitives::f32_buffer::f32_avg_impl));

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
