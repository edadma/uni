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
    use crate::compat::Rc;

    // Helper to add async builtin with optional documentation
    let add_builtin = |interp: &mut AsyncInterpreter, name: &str, func: AsyncBuiltinFn, doc: Option<&str>| {
        let atom = interp.intern_atom(name);
        interp.dictionary.insert(
            atom,
            DictEntry {
                value: Value::AsyncBuiltin(func),
                is_executable: true,
                doc: doc.map(|s| Rc::<str>::from(s)),
            },
        );
    };

    // Async I/O primitives
    add_builtin(interp, ".", crate::primitives::print::print_builtin,
        Some("Print the top stack item.\nUsage: value . => (prints value)\nExample: 42 . => 42"));
    add_builtin(interp, "emit", crate::primitives::emit::emit_builtin,
        Some("Output a character from its numeric code.\nUsage: code emit\nExample: 65 emit => A"));
    add_builtin(interp, "words", crate::primitives::words::words_builtin,
        Some("List all defined words in the dictionary.\nUsage: words"));
    add_builtin(interp, "space", crate::primitives::space::space_builtin,
        Some("Output a space character.\nUsage: space"));
    // Note: cr is now defined in the prelude as [10 emit]

    // Async concurrency primitives
    add_builtin(interp, "delay", crate::primitives::delay::delay,
        Some("Wait for N milliseconds while letting other tasks run.\nUsage: ms delay\nExample: 1000 delay => (waits 1 second)"));
    // TODO: Implement spawn once we understand quotation/list representation
    // add_builtin(interp, "spawn", crate::primitives::spawn::spawn,
    //     Some("Spawn a quotation as a background task.\nUsage: [code] spawn\nExample: [\"tick\" . cr 1000 delay] spawn"));

    // Utility primitives
    add_builtin(interp, "help", crate::primitives::help::help_builtin,
        Some("Display help for a word.\nUsage: 'word help\nExample: '+ help"));
    add_builtin(interp, "stack", crate::primitives::stack::stack_builtin,
        Some("Display the current stack contents.\nUsage: stack"));
    add_builtin(interp, "clear", sync_builtin!(crate::primitives::clear::clear_impl),
        Some("Clear all items from the stack.\nUsage: clear"));

    // Sync stack primitives (wrapped in async)
    // Note: swap, dup, over, rot are defined in the prelude using pick and roll
    add_builtin(interp, "drop", sync_builtin!(crate::primitives::stack::drop_impl),
        Some("Remove the top stack item.\nUsage: value drop\nExample: 42 drop => (empty stack)"));
    add_builtin(interp, "pick", sync_builtin!(crate::primitives::pick::pick_impl),
        Some("Copy the nth item from the stack to the top.\nUsage: ... n pick\nExample: 1 2 3 1 pick => 1 2 3 2"));
    add_builtin(interp, "roll", sync_builtin!(crate::primitives::roll::roll_impl),
        Some("Move the nth item to the top of the stack.\nUsage: ... n roll\nExample: 1 2 3 2 roll => 2 3 1"));

    // Return stack primitives
    add_builtin(interp, ">r", sync_builtin!(crate::primitives::return_stack::to_r_impl),
        Some("Move top item to return stack.\nUsage: value >r"));
    add_builtin(interp, "r>", sync_builtin!(crate::primitives::return_stack::from_r_impl),
        Some("Move top item from return stack to data stack.\nUsage: r>"));
    add_builtin(interp, "r@", sync_builtin!(crate::primitives::return_stack::r_fetch_impl),
        Some("Copy top return stack item to data stack.\nUsage: r@"));

    // Sync arithmetic primitives (wrapped in async)
    add_builtin(interp, "+", sync_builtin!(crate::primitives::plus::add_impl),
        Some("Add two numbers or concatenate strings.\nUsage: a b + => result\nExamples: 5 3 + => 8, \"Hello \" \"World\" + => \"Hello World\""));
    add_builtin(interp, "-", sync_builtin!(crate::primitives::minus::sub_impl),
        Some("Subtract two numbers.\nUsage: a b - => result\nExample: 10 3 - => 7"));
    add_builtin(interp, "*", sync_builtin!(crate::primitives::multiply::mul_impl),
        Some("Multiply two numbers.\nUsage: a b * => result\nExample: 6 7 * => 42"));
    add_builtin(interp, "/", sync_builtin!(crate::primitives::divide::div_impl),
        Some("Divide two numbers.\nUsage: a b / => result\nExample: 15 3 / => 5"));
    add_builtin(interp, "mod", sync_builtin!(crate::primitives::modulo::mod_impl),
        Some("Modulo operation.\nUsage: a b mod => remainder\nExample: 13 5 mod => 3"));
    add_builtin(interp, "//", sync_builtin!(crate::primitives::floor_div::floor_div_impl),
        Some("Floor division.\nUsage: a b // => quotient\nExample: 7 2 // => 3"));
    add_builtin(interp, "div", sync_builtin!(crate::primitives::trunc_div::trunc_div_impl),
        Some("Truncating division.\nUsage: a b div => quotient\nExample: -7 2 div => -3"));
    add_builtin(interp, "abs", sync_builtin!(crate::primitives::abs::abs_impl),
        Some("Absolute value.\nUsage: n abs => |n|\nExample: -5 abs => 5"));
    add_builtin(interp, "min", sync_builtin!(crate::primitives::min::min_impl),
        Some("Minimum of two numbers.\nUsage: a b min => min\nExample: 3 7 min => 3"));
    add_builtin(interp, "max", sync_builtin!(crate::primitives::max::max_impl),
        Some("Maximum of two numbers.\nUsage: a b max => max\nExample: 3 7 max => 7"));

    // Sync comparison primitives (wrapped in async)
    add_builtin(interp, "=", sync_builtin!(crate::primitives::equals::equals_impl),
        Some("Test equality.\nUsage: a b = => bool\nExample: 5 5 = => true"));
    add_builtin(interp, "!=", sync_builtin!(crate::primitives::not_equal::not_equal_impl),
        Some("Test inequality.\nUsage: a b != => bool\nExample: 5 3 != => true"));
    add_builtin(interp, "<", sync_builtin!(crate::primitives::less_than::less_than_impl),
        Some("Test less than.\nUsage: a b < => bool\nExample: 3 7 < => true"));
    add_builtin(interp, ">", sync_builtin!(crate::primitives::greater_than::greater_than_impl),
        Some("Test greater than.\nUsage: a b > => bool\nExample: 7 3 > => true"));
    add_builtin(interp, "<=", sync_builtin!(crate::primitives::less_equal::less_equal_impl),
        Some("Test less than or equal.\nUsage: a b <= => bool\nExample: 3 3 <= => true"));
    add_builtin(interp, ">=", sync_builtin!(crate::primitives::greater_equal::greater_equal_impl),
        Some("Test greater than or equal.\nUsage: a b >= => bool\nExample: 7 7 >= => true"));

    // Sync definition primitives (wrapped in async)
    add_builtin(interp, "def", sync_builtin!(crate::primitives::def::def_impl),
        Some("Define an executable word.\nUsage: 'name [body] def\nExample: 'square [dup *] def"));
    add_builtin(interp, "val", sync_builtin!(crate::primitives::val::val_impl),
        Some("Define a constant value.\nUsage: 'name value val\nExample: 'pi 3.14159 val"));
    add_builtin(interp, "doc", sync_builtin!(crate::primitives::doc::doc_impl),
        Some("Add documentation to the last defined word.\nUsage: \"documentation\" doc"));

    // Variable primitives
    add_builtin(interp, "var", sync_builtin!(crate::primitives::var::var_impl),
        Some("Create a mutable variable.\nUsage: initial-value 'name var\nExample: 0 'counter var"));
    add_builtin(interp, "@", sync_builtin!(crate::primitives::fetch::fetch_impl),
        Some("Fetch value from a variable.\nUsage: var @ => value\nExample: counter @ => 0"));
    add_builtin(interp, "!", sync_builtin!(crate::primitives::store::store_impl),
        Some("Store value in a variable.\nUsage: value var !\nExample: 5 counter !"));
    add_builtin(interp, "lval", sync_builtin!(crate::primitives::lval::lval_impl),
        Some("Define a local constant value.\nUsage: 'name value lval"));
    add_builtin(interp, "lvar", sync_builtin!(crate::primitives::lvar::lvar_impl),
        Some("Create a local mutable variable.\nUsage: initial-value 'name lvar"));

    // Sync list primitives (wrapped in async)
    add_builtin(interp, "cons", sync_builtin!(crate::primitives::cons::cons_impl),
        Some("Construct a pair from two values.\nUsage: head tail cons => (head . tail)\nExample: 1 null cons => (1)"));
    add_builtin(interp, "car", sync_builtin!(crate::primitives::head::car_impl),
        Some("Get the first element of a pair.\nUsage: pair car => head\nExample: (1 2 3) car => 1"));
    add_builtin(interp, "cdr", sync_builtin!(crate::primitives::tail::cdr_impl),
        Some("Get the rest of a pair.\nUsage: pair cdr => tail\nExample: (1 2 3) cdr => (2 3)"));
    add_builtin(interp, "list", sync_builtin!(crate::primitives::list::list_impl),
        Some("Create a list from stack items.\nUsage: n item1 ... itemN list => (item1 ... itemN)\nExample: 3 1 2 3 list => (1 2 3)"));

    // Date/time primitives (wrapped in async)
    add_builtin(interp, "now", sync_builtin!(crate::primitives::now::now_impl),
        Some("Get current date and time as a record.\nUsage: now => date-record\nFields: year month day hour minute second offset"));

    // Advanced math primitives (feature-gated, wrapped in async)
    #[cfg(feature = "advanced_math")]
    {
        add_builtin(interp, "sqrt", sync_builtin!(crate::primitives::sqrt::sqrt_impl),
            Some("Square root.\nUsage: n sqrt => sqrt(n)\nExample: 16 sqrt => 4"));
        add_builtin(interp, "pow", sync_builtin!(crate::primitives::pow::pow_impl),
            Some("Power function.\nUsage: base exponent pow => base^exponent\nExample: 2 10 pow => 1024"));
        add_builtin(interp, "floor", sync_builtin!(crate::primitives::floor::floor_impl),
            Some("Round down to nearest integer.\nUsage: n floor => floor(n)\nExample: 3.7 floor => 3"));
        add_builtin(interp, "ceil", sync_builtin!(crate::primitives::ceil::ceil_impl),
            Some("Round up to nearest integer.\nUsage: n ceil => ceil(n)\nExample: 3.2 ceil => 4"));
        add_builtin(interp, "round", sync_builtin!(crate::primitives::round::round_impl),
            Some("Round to nearest integer.\nUsage: n round => round(n)\nExample: 3.5 round => 4"));
        add_builtin(interp, "sin", sync_builtin!(crate::primitives::sin::sin_impl),
            Some("Sine function (radians).\nUsage: angle sin => sin(angle)\nExample: 0 sin => 0"));
        add_builtin(interp, "cos", sync_builtin!(crate::primitives::cos::cos_impl),
            Some("Cosine function (radians).\nUsage: angle cos => cos(angle)\nExample: 0 cos => 1"));
        add_builtin(interp, "tan", sync_builtin!(crate::primitives::tan::tan_impl),
            Some("Tangent function (radians).\nUsage: angle tan => tan(angle)\nExample: 0 tan => 0"));
        add_builtin(interp, "log", sync_builtin!(crate::primitives::log::log_impl),
            Some("Natural logarithm.\nUsage: n log => ln(n)\nExample: 2.71828 log => 1"));
        add_builtin(interp, "exp", sync_builtin!(crate::primitives::exp::exp_impl),
            Some("Exponential function.\nUsage: n exp => e^n\nExample: 1 exp => 2.71828"));
    }

    // Record primitives (wrapped in async)
    add_builtin(interp, "make-record-type", sync_builtin!(crate::primitives::record::make_record_type_impl),
        Some("Create a new record type.\nUsage: field-list type-name make-record-type => record-type\nExample: (x y) \"point\" make-record-type"));
    add_builtin(interp, "construct-record", sync_builtin!(crate::primitives::record::construct_record_impl),
        Some("Construct a record instance.\nUsage: value-list record-type construct-record => record\nExample: (3 4) point-type construct-record"));
    add_builtin(interp, "is-record-type?", sync_builtin!(crate::primitives::record::is_record_type_impl),
        Some("Check if value is a record type.\nUsage: value is-record-type? => bool"));
    add_builtin(interp, "get-record-field", sync_builtin!(crate::primitives::record::get_record_field_impl),
        Some("Get a field value from a record.\nUsage: record field-name get-record-field => value\nExample: my-point 'x get-record-field => 3"));
    add_builtin(interp, "set-record-field!", sync_builtin!(crate::primitives::record::set_record_field_impl),
        Some("Set a field value in a record.\nUsage: value record field-name set-record-field!\nExample: 5 my-point 'x set-record-field!"));
    add_builtin(interp, "record-type-of", sync_builtin!(crate::primitives::record::record_type_of_impl),
        Some("Get the record type of a record instance.\nUsage: record record-type-of => record-type"));

    // Vector primitives
    add_builtin(interp, "vector", sync_builtin!(crate::primitives::vector::vector_impl),
        Some("Create a vector from stack items.\nUsage: n item1 ... itemN vector => #(item1 ... itemN)\nExample: 3 1 2 3 vector => #(1 2 3)"));
    add_builtin(interp, "make-vector", sync_builtin!(crate::primitives::vector::make_vector_impl),
        Some("Create a vector of specified size with initial value.\nUsage: initial-value size make-vector => vector\nExample: 0 5 make-vector => #(0 0 0 0 0)"));
    add_builtin(interp, "vector-length", sync_builtin!(crate::primitives::vector::vector_length_impl),
        Some("Get the length of a vector.\nUsage: vector vector-length => n\nExample: #(1 2 3) vector-length => 3"));
    add_builtin(interp, "vector-ref", sync_builtin!(crate::primitives::vector::vector_ref_impl),
        Some("Get an element from a vector.\nUsage: vector index vector-ref => value\nExample: #(1 2 3) 1 vector-ref => 2"));
    add_builtin(interp, "vector-set!", sync_builtin!(crate::primitives::vector::vector_set_impl),
        Some("Set an element in a vector.\nUsage: value vector index vector-set!\nExample: 5 my-vec 0 vector-set!"));
    add_builtin(interp, "vector->list", sync_builtin!(crate::primitives::vector::vector_to_list_impl),
        Some("Convert a vector to a list.\nUsage: vector vector->list => list\nExample: #(1 2 3) vector->list => (1 2 3)"));
    add_builtin(interp, "list->vector", sync_builtin!(crate::primitives::vector::list_to_vector_impl),
        Some("Convert a list to a vector.\nUsage: list list->vector => vector\nExample: (1 2 3) list->vector => #(1 2 3)"));

    // Bitwise primitives
    add_builtin(interp, "&", sync_builtin!(crate::primitives::bit_and::bit_and_impl),
        Some("Bitwise AND.\nUsage: a b & => result\nExample: 12 10 & => 8"));
    add_builtin(interp, "|", sync_builtin!(crate::primitives::bit_or::bit_or_impl),
        Some("Bitwise OR.\nUsage: a b | => result\nExample: 12 10 | => 14"));
    add_builtin(interp, "^", sync_builtin!(crate::primitives::bit_xor::bit_xor_impl),
        Some("Bitwise XOR.\nUsage: a b ^ => result\nExample: 12 10 ^ => 6"));
    add_builtin(interp, "~", sync_builtin!(crate::primitives::bit_not::bit_not_impl),
        Some("Bitwise NOT.\nUsage: n ~ => result\nExample: 5 ~ => -6"));
    add_builtin(interp, "<<", sync_builtin!(crate::primitives::shl::shl_impl),
        Some("Bitwise left shift.\nUsage: value shift << => result\nExample: 3 2 << => 12"));
    add_builtin(interp, ">>", sync_builtin!(crate::primitives::shr::shr_impl),
        Some("Bitwise right shift.\nUsage: value shift >> => result\nExample: 12 2 >> => 3"));

    // String/type primitives
    add_builtin(interp, "->string", sync_builtin!(crate::primitives::to_string::to_string_impl),
        Some("Convert value to string.\nUsage: value ->string => string\nExample: 42 ->string => \"42\""));
    add_builtin(interp, "truthy?", sync_builtin!(crate::primitives::truthy::truthy_impl),
        Some("Test if value is truthy (not false or null).\nUsage: value truthy? => bool\nExample: 0 truthy? => true"));
    add_builtin(interp, "type-of", sync_builtin!(crate::primitives::type_of::type_of_impl),
        Some("Get the type of a value.\nUsage: value type-of => type-string\nExample: 42 type-of => \"i64\""));

    // I32 buffer primitives (for integer data and DSP)
    add_builtin(interp, "i32-buffer", sync_builtin!(crate::primitives::i32_buffer::i32_buffer_impl),
        Some("Create an i32 buffer.\nUsage: capacity i32-buffer => buffer\nExample: 100 i32-buffer"));
    add_builtin(interp, "i32@", sync_builtin!(crate::primitives::i32_buffer::i32_ref_impl),
        Some("Get value from i32 buffer.\nUsage: buffer index i32@ => value\nExample: buf 0 i32@ => 42"));
    add_builtin(interp, "i32!", sync_builtin!(crate::primitives::i32_buffer::i32_set_impl),
        Some("Set value in i32 buffer.\nUsage: value buffer index i32!\nExample: 42 buf 0 i32!"));
    add_builtin(interp, "i32-length", sync_builtin!(crate::primitives::i32_buffer::i32_length_impl),
        Some("Get length of i32 buffer.\nUsage: buffer i32-length => n\nExample: buf i32-length => 100"));
    add_builtin(interp, "i32-push!", sync_builtin!(crate::primitives::i32_buffer::i32_push_impl),
        Some("Push value to i32 buffer.\nUsage: value buffer i32-push!\nExample: 42 buf i32-push!"));
    add_builtin(interp, "i32-pop!", sync_builtin!(crate::primitives::i32_buffer::i32_pop_impl),
        Some("Pop value from i32 buffer.\nUsage: buffer i32-pop! => value\nExample: buf i32-pop! => 42"));
    add_builtin(interp, "i32-max", sync_builtin!(crate::primitives::i32_buffer::i32_max_impl),
        Some("Find maximum value in i32 buffer.\nUsage: buffer i32-max => max\nExample: buf i32-max => 100"));
    add_builtin(interp, "i32-min", sync_builtin!(crate::primitives::i32_buffer::i32_min_impl),
        Some("Find minimum value in i32 buffer.\nUsage: buffer i32-min => min\nExample: buf i32-min => -50"));
    add_builtin(interp, "i32-avg", sync_builtin!(crate::primitives::i32_buffer::i32_avg_impl),
        Some("Calculate average of i32 buffer.\nUsage: buffer i32-avg => avg\nExample: buf i32-avg => 25"));

    // F32 buffer primitives (for float data, audio/DSP, GPU compute)
    add_builtin(interp, "f32-buffer", sync_builtin!(crate::primitives::f32_buffer::f32_buffer_impl),
        Some("Create an f32 buffer.\nUsage: capacity f32-buffer => buffer\nExample: 100 f32-buffer"));
    add_builtin(interp, "f32@", sync_builtin!(crate::primitives::f32_buffer::f32_ref_impl),
        Some("Get value from f32 buffer.\nUsage: buffer index f32@ => value\nExample: buf 0 f32@ => 3.14"));
    add_builtin(interp, "f32!", sync_builtin!(crate::primitives::f32_buffer::f32_set_impl),
        Some("Set value in f32 buffer.\nUsage: value buffer index f32!\nExample: 3.14 buf 0 f32!"));
    add_builtin(interp, "f32-length", sync_builtin!(crate::primitives::f32_buffer::f32_length_impl),
        Some("Get length of f32 buffer.\nUsage: buffer f32-length => n\nExample: buf f32-length => 100"));
    add_builtin(interp, "f32-push!", sync_builtin!(crate::primitives::f32_buffer::f32_push_impl),
        Some("Push value to f32 buffer.\nUsage: value buffer f32-push!\nExample: 3.14 buf f32-push!"));
    add_builtin(interp, "f32-pop!", sync_builtin!(crate::primitives::f32_buffer::f32_pop_impl),
        Some("Pop value from f32 buffer.\nUsage: buffer f32-pop! => value\nExample: buf f32-pop! => 3.14"));
    add_builtin(interp, "f32-max", sync_builtin!(crate::primitives::f32_buffer::f32_max_impl),
        Some("Find maximum value in f32 buffer.\nUsage: buffer f32-max => max\nExample: buf f32-max => 100.5"));
    add_builtin(interp, "f32-min", sync_builtin!(crate::primitives::f32_buffer::f32_min_impl),
        Some("Find minimum value in f32 buffer.\nUsage: buffer f32-min => min\nExample: buf f32-min => -50.3"));
    add_builtin(interp, "f32-avg", sync_builtin!(crate::primitives::f32_buffer::f32_avg_impl),
        Some("Calculate average of f32 buffer.\nUsage: buffer f32-avg => avg\nExample: buf f32-avg => 25.7"));

    // Create the date record type for use by the 'now' primitive
    // Field names: year month day hour minute second offset
    create_date_record_type(interp);
}

// Helper function to create the date record type used by 'now'
fn create_date_record_type(interp: &mut AsyncInterpreter) {
    use crate::compat::{Rc, vec};
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
