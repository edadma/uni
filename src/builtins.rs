use crate::compat::Rc;
use crate::interpreter::Interpreter;
use crate::value::Value;

use crate::primitives::{
    // Basic math functions
    abs_builtin,
    // Basic arithmetic
    add_builtin,
    // Bitwise operations
    bit_and_builtin,
    bit_not_builtin,
    bit_or_builtin,
    bit_xor_builtin,
    // List operations
    cons_builtin,
    construct_record_builtin,
    // Meta operations
    def_builtin,
    div_builtin,
    doc_builtin,
    // Stack operations
    drop_builtin,
    eq_builtin,
    floor_div_builtin,
    from_r_builtin,
    get_record_field_builtin,
    greater_equal_builtin,
    greater_than_builtin,
    head_builtin,
    help_builtin,
    is_record_type_builtin,
    less_equal_builtin,
    // Comparison operations
    less_than_builtin,
    list_builtin,
    list_to_vector_builtin,
    make_record_type_builtin,
    make_vector_builtin,
    max_builtin,
    min_builtin,
    mod_builtin,
    mul_builtin,
    not_equal_builtin,
    null_predicate_builtin,
    pick_builtin,
    // Control flow - if is now special in evaluator
    // I/O operations
    print_builtin,
    words_builtin,
    // Stack management
    clear_builtin,
    stack_builtin,
    r_fetch_builtin,
    record_type_of_builtin,
    roll_builtin,
    set_record_field_builtin,
    // Shift operations
    shl_builtin,
    shr_builtin,
    sub_builtin,
    tail_builtin,
    // Return stack operations
    to_r_builtin,
    // String operations
    to_string_builtin,
    trunc_div_builtin,
    // Predicate operations
    truthy_predicate_builtin,
    // Type introspection
    type_of_builtin,
    val_builtin,
    vector_builtin,
    vector_length_builtin,
    vector_ref_builtin,
    vector_set_builtin,
    vector_to_list_builtin,
};

// Advanced math operations (only with advanced_math feature)
#[cfg(feature = "advanced_math")]
use crate::primitives::{
    ceil_builtin,
    cos_builtin,
    exp_builtin,
    floor_builtin,
    log_builtin,
    pow_builtin,
    round_builtin,
    sin_builtin,
    sqrt_builtin,
    tan_builtin,
};

// Date/time operations (only with datetime feature)
#[cfg(feature = "datetime")]
use crate::primitives::{
    date_add_builtin, date_equal_builtin, date_greater_than_builtin, date_less_than_builtin,
    date_sub_builtin, datetime_builtin, datetime_to_string_builtin,
    datetime_with_offset_builtin, day_builtin, duration_builtin, duration_equal_builtin,
    duration_greater_than_builtin, duration_less_than_builtin, duration_to_seconds_builtin,
    hour_builtin, minute_builtin, month_builtin, now_builtin, second_builtin,
    string_to_datetime_builtin, timestamp_builtin, timestamp_to_datetime_builtin,
    to_local_builtin, to_utc_builtin, weekday_builtin, year_builtin,
};

// Hardware operations (only on micro:bit)
#[cfg(target_os = "none")]
use crate::primitives::button_read_builtin;

// I16 buffer operations (audio/DSP)
use crate::primitives::{
    i16_avg_builtin, i16_buffer_builtin, i16_length_builtin, i16_max_builtin, i16_min_builtin,
    i16_pop_builtin, i16_push_builtin, i16_ref_builtin, i16_set_builtin,
};

// RUST CONCEPT: Registering all builtins with the interpreter
pub fn register_builtins(interp: &mut Interpreter) {
    use crate::interpreter::DictEntry;

    // Arithmetic operations
    let add_atom = interp.intern_atom("+");
    interp.dictionary.insert(
        add_atom,
        DictEntry {
            value: Value::Builtin(add_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Add two numbers or concatenate strings.\nUsage: a b + => result\nExamples: 5 3 + => 8\n\"Hello \" \"World\" + => \"Hello World\"",
            )),
        },
    );

    let sub_atom = interp.intern_atom("-");
    interp.dictionary.insert(
        sub_atom,
        DictEntry {
            value: Value::Builtin(sub_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Subtract two numbers.\nUsage: a b - => result\nExample: 10 3 - => 7",
            )),
        },
    );

    let mul_atom = interp.intern_atom("*");
    interp.dictionary.insert(
        mul_atom,
        DictEntry {
            value: Value::Builtin(mul_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Multiply two numbers.\nUsage: a b * => result\nExample: 6 7 * => 42",
            )),
        },
    );

    let div_atom = interp.intern_atom("/");
    interp.dictionary.insert(
        div_atom,
        DictEntry {
            value: Value::Builtin(div_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Divide two numbers.\nUsage: a b / => result\nExample: 15 3 / => 5",
            )),
        },
    );

    let floor_div_atom = interp.intern_atom("//");
    interp.dictionary.insert(
        floor_div_atom,
        DictEntry {
            value: Value::Builtin(floor_div_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Floor division (like Python's //).\nUsage: a b // => result\nExample: 7 2 // => 3, -7 2 // => -4",
            )),
        },
    );

    let mod_atom = interp.intern_atom("mod");
    interp.dictionary.insert(
        mod_atom,
        DictEntry {
            value: Value::Builtin(mod_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Calculate modulo (remainder after division).\nUsage: a b mod => remainder\nExample: 10 3 mod => 1",
            )),
        },
    );

    let trunc_div_atom = interp.intern_atom("div");
    interp.dictionary.insert(
        trunc_div_atom,
        DictEntry {
            value: Value::Builtin(trunc_div_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Truncating integer division (rounds toward zero).\nUsage: a b div => result\nExample: 7 2 div => 3, -7 2 div => -3",
            )),
        },
    );

    let eq_atom = interp.intern_atom("=");
    interp.dictionary.insert(
        eq_atom,
        DictEntry {
            value: Value::Builtin(eq_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Test equality of two values.\nUsage: a b = => boolean\nExample: 5 5 = => true",
            )),
        },
    );

    // Stack operations
    let roll_atom = interp.intern_atom("roll");
    interp.dictionary.insert(
        roll_atom,
        DictEntry {
            value: Value::Builtin(roll_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Rotate n-th stack item to top.\nUsage: x1 x2 ... xn n roll => x2 ... xn x1\nExample: 1 2 3 2 roll => 2 3 1",
            )),
        },
    );

    let pick_atom = interp.intern_atom("pick");
    interp.dictionary.insert(
        pick_atom,
        DictEntry {
            value: Value::Builtin(pick_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Copy n-th stack item to top.\nUsage: x1 x2 ... xn n pick => x1 x2 ... xn x1\nExample: 1 2 3 2 pick => 1 2 3 1",
            )),
        },
    );

    let drop_atom = interp.intern_atom("drop");
    interp.dictionary.insert(
        drop_atom,
        DictEntry {
            value: Value::Builtin(drop_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Remove top stack item.\nUsage: x drop =>\nExample: 1 2 3 drop => 1 2",
            )),
        },
    );

    // NOTE: exec and if are now handled specially in the evaluator, not as builtins

    // Dictionary operations
    let def_atom = interp.intern_atom("def");
    interp.dictionary.insert(
        def_atom,
        DictEntry {
            value: Value::Builtin(def_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Define an executable word. Usage: 'name body def",
            )),
        },
    );

    let val_atom = interp.intern_atom("val");
    interp.dictionary.insert(
        val_atom,
        DictEntry {
            value: Value::Builtin(val_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Define a constant value. Usage: 'name value val",
            )),
        },
    );

    let doc_atom = interp.intern_atom("doc");
    interp.dictionary.insert(
        doc_atom,
        DictEntry {
            value: Value::Builtin(doc_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Attach documentation to the most recent def/val.\nUsage: \"lines\" doc",
            )),
        },
    );

    let help_atom = interp.intern_atom("help");
    interp.dictionary.insert(
        help_atom,
        DictEntry {
            value: Value::Builtin(help_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Display documentation for a word. Usage: 'name help",
            )),
        },
    );

    // I/O operations
    let print_atom = interp.intern_atom(".");
    interp.dictionary.insert(
        print_atom,
        DictEntry {
            value: Value::Builtin(print_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Print top stack value to output.\nUsage: value . => (prints value)\nExample: \"Hello\" . => Hello",
            )),
        },
    );

    let words_atom = interp.intern_atom("words");
    interp.dictionary.insert(
        words_atom,
        DictEntry {
            value: Value::Builtin(words_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Display all defined words in the dictionary.\nUsage: words => (displays all words)\nExample: words => Defined words (120): ...",
            )),
        },
    );

    // Stack management operations
    let stack_atom = interp.intern_atom("stack");
    interp.dictionary.insert(
        stack_atom,
        DictEntry {
            value: Value::Builtin(stack_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Display the current stack contents.\nUsage: stack => (displays stack)\nExample: 1 2 3 stack => Stack (3 items): ...",
            )),
        },
    );

    let clear_atom = interp.intern_atom("clear");
    interp.dictionary.insert(
        clear_atom,
        DictEntry {
            value: Value::Builtin(clear_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Clear all items from the stack.\nUsage: clear => (empties stack)\nExample: 1 2 3 clear stack => Stack is empty",
            )),
        },
    );

    // Hardware operations (micro:bit only)
    #[cfg(target_os = "none")]
    {
    let button_read_atom = interp.intern_atom("button-read");
    interp.dictionary.insert(
        button_read_atom,
        DictEntry {
            value: Value::Builtin(button_read_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Read button state on micro:bit.\nUsage: button-id button-read => boolean\nExample: 0 button-read => true (button A pressed)\n0=A, 1=B",
            )),
        },
    );
    }

    // I16 buffer operations (audio/DSP)
    let i16_buffer_atom = interp.intern_atom("i16-buffer");
    interp.dictionary.insert(
        i16_buffer_atom,
        DictEntry {
            value: Value::Builtin(i16_buffer_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Create i16 buffer of specified size.\nUsage: size i16-buffer => buffer\nExample: 1024 i16-buffer => #<i16-buffer:1024:[0 0 0 0 0 0 0 0 ...]>",
            )),
        },
    );

    let i16_ref_atom = interp.intern_atom("i16-ref");
    interp.dictionary.insert(
        i16_ref_atom,
        DictEntry {
            value: Value::Builtin(i16_ref_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Get value at index from i16 buffer.\nUsage: buffer index i16-ref => value\nExample: buffer 0 i16-ref => 100",
            )),
        },
    );

    let i16_set_atom = interp.intern_atom("i16-set!");
    interp.dictionary.insert(
        i16_set_atom,
        DictEntry {
            value: Value::Builtin(i16_set_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Set value at index in i16 buffer.\nUsage: value buffer index i16-set! => buffer\nExample: 100 buffer 0 i16-set! => buffer",
            )),
        },
    );

    let i16_length_atom = interp.intern_atom("i16-length");
    interp.dictionary.insert(
        i16_length_atom,
        DictEntry {
            value: Value::Builtin(i16_length_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Get length of i16 buffer.\nUsage: buffer i16-length => buffer length\nExample: buffer i16-length => buffer 1024",
            )),
        },
    );

    let i16_push_atom = interp.intern_atom("i16-push!");
    interp.dictionary.insert(
        i16_push_atom,
        DictEntry {
            value: Value::Builtin(i16_push_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Append value to end of i16 buffer.\nUsage: value buffer i16-push! => buffer\nExample: 100 buffer i16-push! => buffer",
            )),
        },
    );

    let i16_pop_atom = interp.intern_atom("i16-pop!");
    interp.dictionary.insert(
        i16_pop_atom,
        DictEntry {
            value: Value::Builtin(i16_pop_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Remove and return last value from i16 buffer.\nUsage: buffer i16-pop! => buffer value\nExample: buffer i16-pop! => buffer 100",
            )),
        },
    );

    let i16_max_atom = interp.intern_atom("i16-max");
    interp.dictionary.insert(
        i16_max_atom,
        DictEntry {
            value: Value::Builtin(i16_max_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Find maximum value in i16 buffer.\nUsage: buffer i16-max => buffer max-value\nExample: buffer i16-max => buffer 1000",
            )),
        },
    );

    let i16_min_atom = interp.intern_atom("i16-min");
    interp.dictionary.insert(
        i16_min_atom,
        DictEntry {
            value: Value::Builtin(i16_min_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Find minimum value in i16 buffer.\nUsage: buffer i16-min => buffer min-value\nExample: buffer i16-min => buffer -500",
            )),
        },
    );

    let i16_avg_atom = interp.intern_atom("i16-avg");
    interp.dictionary.insert(
        i16_avg_atom,
        DictEntry {
            value: Value::Builtin(i16_avg_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Compute average value of i16 buffer.\nUsage: buffer i16-avg => buffer average\nExample: buffer i16-avg => buffer 250",
            )),
        },
    );

    // String operations
    let to_string_atom = interp.intern_atom("->string");
    interp.dictionary.insert(
        to_string_atom,
        DictEntry {
            value: Value::Builtin(to_string_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Convert value to string representation.\nUsage: value ->string => string\nExample: 42 ->string => \"42\"",
            )),
        },
    );

    // List operations
    let head_atom = interp.intern_atom("head");
    interp.dictionary.insert(
        head_atom,
        DictEntry {
            value: Value::Builtin(head_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Get first element of a list.\nUsage: [x y z] head => x\nExample: [1 2 3] head => 1",
            )),
        },
    );

    let tail_atom = interp.intern_atom("tail");
    interp.dictionary.insert(
        tail_atom,
        DictEntry {
            value: Value::Builtin(tail_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Get all but first element of a list.\nUsage: [x y z] tail => [y z]\nExample: [1 2 3] tail => [2 3]",
            )),
        },
    );

    let cons_atom = interp.intern_atom("cons");
    interp.dictionary.insert(
        cons_atom,
        DictEntry {
            value: Value::Builtin(cons_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Prepend element to list (construct cons cell).\nUsage: x [y z] cons => [x y z]\nExample: 1 [2 3] cons => [1 2 3]",
            )),
        },
    );

    let list_atom = interp.intern_atom("list");
    interp.dictionary.insert(
        list_atom,
        DictEntry {
            value: Value::Builtin(list_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Convert all stack items into a list.\nUsage: x y z n list => [x y z]\nExample: 1 2 3 3 list => [1 2 3]",
            )),
        },
    );

    let vector_atom = interp.intern_atom("vector");
    interp.dictionary.insert(
        vector_atom,
        DictEntry {
            value: Value::Builtin(vector_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Create vector from stack items.\nUsage: x y z n vector => #[x y z]\nExample: 1 2 3 3 vector => #[1 2 3]",
            )),
        },
    );

    let make_vector_atom = interp.intern_atom("make-vector");
    interp.dictionary.insert(
        make_vector_atom,
        DictEntry {
            value: Value::Builtin(make_vector_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Create vector of size n filled with value.\nUsage: n value make-vector => #[value ...]\nExample: 3 0 make-vector => #[0 0 0]",
            )),
        },
    );

    let vector_length_atom = interp.intern_atom("vector-length");
    interp.dictionary.insert(
        vector_length_atom,
        DictEntry {
            value: Value::Builtin(vector_length_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Get length of a vector.\nUsage: #[a b c] vector-length => 3\nExample: #[1 2 3] vector-length => 3",
            )),
        },
    );

    let vector_ref_atom = interp.intern_atom("vector-ref");
    interp.dictionary.insert(
        vector_ref_atom,
        DictEntry {
            value: Value::Builtin(vector_ref_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Get element at index from vector.\nUsage: #[a b c] i vector-ref => element\nExample: #[10 20 30] 1 vector-ref => 20",
            )),
        },
    );

    let vector_set_atom = interp.intern_atom("vector-set!");
    interp.dictionary.insert(
        vector_set_atom,
        DictEntry {
            value: Value::Builtin(vector_set_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Set element at index in vector.\nUsage: value #[a b c] i vector-set! => #[a value c]\nExample: 99 #[10 20 30] 1 vector-set! => #[10 99 30]",
            )),
        },
    );

    let vector_to_list_atom = interp.intern_atom("vector->list");
    interp.dictionary.insert(
        vector_to_list_atom,
        DictEntry {
            value: Value::Builtin(vector_to_list_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Convert vector to list.\nUsage: #[a b c] vector->list => [a b c]\nExample: #[1 2 3] vector->list => [1 2 3]",
            )),
        },
    );

    let list_to_vector_atom = interp.intern_atom("list->vector");
    interp.dictionary.insert(
        list_to_vector_atom,
        DictEntry {
            value: Value::Builtin(list_to_vector_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Convert list to vector.\nUsage: [a b c] list->vector => #[a b c]\nExample: [1 2 3] list->vector => #[1 2 3]",
            )),
        },
    );

    // Type checking predicates
    let null_predicate_atom = interp.intern_atom("null?");
    interp.dictionary.insert(
        null_predicate_atom,
        DictEntry {
            value: Value::Builtin(null_predicate_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Test if value is null.\nUsage: value null? => boolean\nExample: null null? => true",
            )),
        },
    );

    let truthy_predicate_atom = interp.intern_atom("truthy?");
    interp.dictionary.insert(
        truthy_predicate_atom,
        DictEntry {
            value: Value::Builtin(truthy_predicate_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Test if value is truthy (non-null, non-false, non-zero).\nUsage: value truthy? => boolean\nExample: 5 truthy? => true",
            )),
        },
    );

    let type_of_atom = interp.intern_atom("type");
    interp.dictionary.insert(
        type_of_atom,
        DictEntry {
            value: Value::Builtin(type_of_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "( x -- string ) Return type name of value as a string.\nTypes: number, integer, rational, gaussian, complex, atom, string, boolean, null, list, vector, nil, builtin",
            )),
        },
    );

    // Comparison operations
    let less_than_atom = interp.intern_atom("<");
    interp.dictionary.insert(
        less_than_atom,
        DictEntry {
            value: Value::Builtin(less_than_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Test if first number is less than second.\nUsage: a b < => boolean\nExample: 3 5 < => true",
            )),
        },
    );

    let greater_than_atom = interp.intern_atom(">");
    interp.dictionary.insert(
        greater_than_atom,
        DictEntry {
            value: Value::Builtin(greater_than_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Test if first number is greater than second.\nUsage: a b > => boolean\nExample: 5 3 > => true",
            )),
        },
    );

    let less_equal_atom = interp.intern_atom("<=");
    interp.dictionary.insert(
        less_equal_atom,
        DictEntry {
            value: Value::Builtin(less_equal_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Test if first number is less than or equal to second.\nUsage: a b <= => boolean\nExample: 3 3 <= => true",
            )),
        },
    );

    let greater_equal_atom = interp.intern_atom(">=");
    interp.dictionary.insert(
        greater_equal_atom,
        DictEntry {
            value: Value::Builtin(greater_equal_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Test if first number is greater than or equal to second.\nUsage: a b >= => boolean\nExample: 5 5 >= => true",
            )),
        },
    );

    let not_equal_atom = interp.intern_atom("!=");
    interp.dictionary.insert(
        not_equal_atom,
        DictEntry {
            value: Value::Builtin(not_equal_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Test if two values are not equal.\nUsage: a b != => boolean\nExample: 5 3 != => true",
            )),
        },
    );

    // Basic math functions
    let abs_atom = interp.intern_atom("abs");
    interp.dictionary.insert(
        abs_atom,
        DictEntry {
            value: Value::Builtin(abs_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Calculate absolute value.\nUsage: n abs => |n|\nExample: -5 abs => 5",
            )),
        },
    );

    let min_atom = interp.intern_atom("min");
    interp.dictionary.insert(
        min_atom,
        DictEntry {
            value: Value::Builtin(min_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Return minimum of two numbers.\nUsage: a b min => min(a,b)\nExample: 3 7 min => 3",
            )),
        },
    );

    let max_atom = interp.intern_atom("max");
    interp.dictionary.insert(
        max_atom,
        DictEntry {
            value: Value::Builtin(max_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Return maximum of two numbers.\nUsage: a b max => max(a,b)\nExample: 3 7 max => 7",
            )),
        },
    );

    #[cfg(feature = "advanced_math")]
    {
    let sqrt_atom = interp.intern_atom("sqrt");
    interp.dictionary.insert(
        sqrt_atom,
        DictEntry {
            value: Value::Builtin(sqrt_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Calculate square root.\nUsage: n sqrt => √n\nExample: 16 sqrt => 4",
            )),
        },
    );
    }

    // Advanced math functions
    #[cfg(feature = "advanced_math")]
    {
    let pow_atom = interp.intern_atom("pow");
    interp.dictionary.insert(
        pow_atom,
        DictEntry {
            value: Value::Builtin(pow_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Raise number to power.\nUsage: base exponent pow => base^exponent\nExample: 2 8 pow => 256",
            )),
        },
    );
    }

    #[cfg(feature = "advanced_math")]
    {
    let floor_atom = interp.intern_atom("floor");
    interp.dictionary.insert(
        floor_atom,
        DictEntry {
            value: Value::Builtin(floor_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Round down to nearest integer.\nUsage: n floor => ⌊n⌋\nExample: 3.7 floor => 3",
            )),
        },
    );
    }

    #[cfg(feature = "advanced_math")]
    {
    let ceil_atom = interp.intern_atom("ceil");
    interp.dictionary.insert(
        ceil_atom,
        DictEntry {
            value: Value::Builtin(ceil_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Round up to nearest integer.\nUsage: n ceil => ⌈n⌉\nExample: 3.2 ceil => 4",
            )),
        },
    );
    }

    #[cfg(feature = "advanced_math")]
    {
    let round_atom = interp.intern_atom("round");
    interp.dictionary.insert(
        round_atom,
        DictEntry {
            value: Value::Builtin(round_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Round to nearest integer.\nUsage: n round => round(n)\nExample: 3.5 round => 4",
            )),
        },
    );
    }

    // Trigonometric functions
    #[cfg(feature = "advanced_math")]
    {
    let sin_atom = interp.intern_atom("sin");
    interp.dictionary.insert(
        sin_atom,
        DictEntry {
            value: Value::Builtin(sin_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Calculate sine (radians).\nUsage: radians sin => sin(radians)\nExample: 0 sin => 0",
            )),
        },
    );
    }

    #[cfg(feature = "advanced_math")]
    {
    let cos_atom = interp.intern_atom("cos");
    interp.dictionary.insert(
        cos_atom,
        DictEntry {
            value: Value::Builtin(cos_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Calculate cosine (radians).\nUsage: radians cos => cos(radians)\nExample: 0 cos => 1",
            )),
        },
    );
    }

    #[cfg(feature = "advanced_math")]
    {
    let tan_atom = interp.intern_atom("tan");
    interp.dictionary.insert(
        tan_atom,
        DictEntry {
            value: Value::Builtin(tan_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Calculate tangent (radians).\nUsage: radians tan => tan(radians)\nExample: 0 tan => 0",
            )),
        },
    );
    }

    // Logarithmic functions
    #[cfg(feature = "advanced_math")]
    {
    let log_atom = interp.intern_atom("log");
    interp.dictionary.insert(
        log_atom,
        DictEntry {
            value: Value::Builtin(log_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Calculate natural logarithm.\nUsage: n log => ln(n)\nExample: 2.718281828 log => 1",
            )),
        },
    );
    }

    #[cfg(feature = "advanced_math")]
    {
    let exp_atom = interp.intern_atom("exp");
    interp.dictionary.insert(
        exp_atom,
        DictEntry {
            value: Value::Builtin(exp_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Calculate e raised to power.\nUsage: n exp => e^n\nExample: 1 exp => 2.718281828",
            )),
        },
    );
    }

    // Bitwise operations
    let bit_and_atom = interp.intern_atom("bit-and");
    interp.dictionary.insert(
        bit_and_atom,
        DictEntry {
            value: Value::Builtin(bit_and_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Bitwise AND of two integers.\nUsage: a b bit-and => a&b\nExample: 12 10 bit-and => 8",
            )),
        },
    );

    let bit_or_atom = interp.intern_atom("bit-or");
    interp.dictionary.insert(
        bit_or_atom,
        DictEntry {
            value: Value::Builtin(bit_or_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Bitwise OR of two integers.\nUsage: a b bit-or => a|b\nExample: 12 10 bit-or => 14",
            )),
        },
    );

    let bit_xor_atom = interp.intern_atom("bit-xor");
    interp.dictionary.insert(
        bit_xor_atom,
        DictEntry {
            value: Value::Builtin(bit_xor_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Bitwise XOR of two integers.\nUsage: a b bit-xor => a^b\nExample: 12 10 bit-xor => 6",
            )),
        },
    );

    let bit_not_atom = interp.intern_atom("bit-not");
    interp.dictionary.insert(
        bit_not_atom,
        DictEntry {
            value: Value::Builtin(bit_not_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Bitwise NOT (complement) of integer.\nUsage: n bit-not => ~n\nExample: 5 bit-not => -6",
            )),
        },
    );

    // Shift operations
    let shl_atom = interp.intern_atom("shl");
    interp.dictionary.insert(
        shl_atom,
        DictEntry {
            value: Value::Builtin(shl_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Shift bits left.\nUsage: n count shl => n<<count\nExample: 5 2 shl => 20",
            )),
        },
    );

    let shr_atom = interp.intern_atom("shr");
    interp.dictionary.insert(
        shr_atom,
        DictEntry {
            value: Value::Builtin(shr_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Shift bits right.\nUsage: n count shr => n>>count\nExample: 20 2 shr => 5",
            )),
        },
    );

    // Return stack operations
    let to_r_atom = interp.intern_atom(">r");
    interp.dictionary.insert(
        to_r_atom,
        DictEntry {
            value: Value::Builtin(to_r_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Move value from data stack to return stack.\nUsage: x >r => (moves x to return stack)\nExample: 5 >r => (5 now on return stack)",
            )),
        },
    );

    let from_r_atom = interp.intern_atom("r>");
    interp.dictionary.insert(
        from_r_atom,
        DictEntry {
            value: Value::Builtin(from_r_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Move value from return stack to data stack.\nUsage: r> => x\nExample: r> => (moves top of return stack to data stack)",
            )),
        },
    );

    let r_fetch_atom = interp.intern_atom("r@");
    interp.dictionary.insert(
        r_fetch_atom,
        DictEntry {
            value: Value::Builtin(r_fetch_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Copy top of return stack to data stack.\nUsage: r@ => x\nExample: r@ => (copies top of return stack without removing it)",
            )),
        },
    );

    // Record operations
    let make_record_type_atom = interp.intern_atom("make-record-type");
    interp.dictionary.insert(
        make_record_type_atom,
        DictEntry {
            value: Value::Builtin(make_record_type_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Create a record type with named fields.\nUsage: [field-names...] \"type-name\" make-record-type => record-type\nExample: [\"name\" \"age\"] \"person\" make-record-type",
            )),
        },
    );

    let construct_record_atom = interp.intern_atom("construct-record");
    interp.dictionary.insert(
        construct_record_atom,
        DictEntry {
            value: Value::Builtin(construct_record_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Internal helper to construct record instances.\nUsage: field-values... n \"type-name\" construct-record => record",
            )),
        },
    );

    let is_record_type_atom = interp.intern_atom("is-record-type?");
    interp.dictionary.insert(
        is_record_type_atom,
        DictEntry {
            value: Value::Builtin(is_record_type_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Check if value is a record of specific type.\nUsage: value \"type-name\" is-record-type? => boolean",
            )),
        },
    );

    let get_record_field_atom = interp.intern_atom("get-record-field");
    interp.dictionary.insert(
        get_record_field_atom,
        DictEntry {
            value: Value::Builtin(get_record_field_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Get field value from record.\nUsage: record \"type-name\" field-index get-record-field => value",
            )),
        },
    );

    let set_record_field_atom = interp.intern_atom("set-record-field!");
    interp.dictionary.insert(
        set_record_field_atom,
        DictEntry {
            value: Value::Builtin(set_record_field_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Set field value in record.\nUsage: new-value record \"type-name\" field-index set-record-field! => record",
            )),
        },
    );

    let record_type_of_atom = interp.intern_atom("record-type-of");
    interp.dictionary.insert(
        record_type_of_atom,
        DictEntry {
            value: Value::Builtin(record_type_of_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Get type name from record instance.\nUsage: record record-type-of => \"type-name\"",
            )),
        },
    );

    // Date/time operations (only with datetime feature)
    #[cfg(feature = "datetime")]
    {
    let now_atom = interp.intern_atom("now");
    interp.dictionary.insert(
        now_atom,
        DictEntry {
            value: Value::Builtin(now_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Get current date/time in local timezone.\nUsage: now => datetime",
            )),
        },
    );

    let datetime_atom = interp.intern_atom("datetime");
    interp.dictionary.insert(
        datetime_atom,
        DictEntry {
            value: Value::Builtin(datetime_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Create datetime in local timezone.\nUsage: year month day hour minute second datetime => datetime\nExample: 2025 10 1 14 30 0 datetime",
            )),
        },
    );

    let datetime_with_offset_atom = interp.intern_atom("datetime-with-offset");
    interp.dictionary.insert(
        datetime_with_offset_atom,
        DictEntry {
            value: Value::Builtin(datetime_with_offset_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Create datetime with specific offset.\nUsage: year month day hour minute second offset-hours datetime-with-offset => datetime\nExample: 2025 10 1 14 30 0 -5 datetime-with-offset",
            )),
        },
    );

    let year_atom = interp.intern_atom("year");
    interp.dictionary.insert(
        year_atom,
        DictEntry {
            value: Value::Builtin(year_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Extract year from datetime.\nUsage: datetime year => year",
            )),
        },
    );

    let month_atom = interp.intern_atom("month");
    interp.dictionary.insert(
        month_atom,
        DictEntry {
            value: Value::Builtin(month_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Extract month (1-12) from datetime.\nUsage: datetime month => month",
            )),
        },
    );

    let day_atom = interp.intern_atom("day");
    interp.dictionary.insert(
        day_atom,
        DictEntry {
            value: Value::Builtin(day_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Extract day (1-31) from datetime.\nUsage: datetime day => day",
            )),
        },
    );

    let hour_atom = interp.intern_atom("hour");
    interp.dictionary.insert(
        hour_atom,
        DictEntry {
            value: Value::Builtin(hour_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Extract hour (0-23) from datetime.\nUsage: datetime hour => hour",
            )),
        },
    );

    let minute_atom = interp.intern_atom("minute");
    interp.dictionary.insert(
        minute_atom,
        DictEntry {
            value: Value::Builtin(minute_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Extract minute (0-59) from datetime.\nUsage: datetime minute => minute",
            )),
        },
    );

    let second_atom = interp.intern_atom("second");
    interp.dictionary.insert(
        second_atom,
        DictEntry {
            value: Value::Builtin(second_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Extract second (0-59) from datetime.\nUsage: datetime second => second",
            )),
        },
    );

    let weekday_atom = interp.intern_atom("weekday");
    interp.dictionary.insert(
        weekday_atom,
        DictEntry {
            value: Value::Builtin(weekday_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Get day of week (0=Monday, 6=Sunday) from datetime.\nUsage: datetime weekday => weekday",
            )),
        },
    );

    let timestamp_atom = interp.intern_atom("timestamp");
    interp.dictionary.insert(
        timestamp_atom,
        DictEntry {
            value: Value::Builtin(timestamp_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Convert datetime to Unix timestamp (seconds since epoch).\nUsage: datetime timestamp => timestamp",
            )),
        },
    );

    let timestamp_to_datetime_atom = interp.intern_atom("timestamp->datetime");
    interp.dictionary.insert(
        timestamp_to_datetime_atom,
        DictEntry {
            value: Value::Builtin(timestamp_to_datetime_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Convert Unix timestamp to datetime in local timezone.\nUsage: timestamp timestamp->datetime => datetime",
            )),
        },
    );

    let to_utc_atom = interp.intern_atom("to-utc");
    interp.dictionary.insert(
        to_utc_atom,
        DictEntry {
            value: Value::Builtin(to_utc_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Convert datetime to UTC.\nUsage: datetime to-utc => datetime",
            )),
        },
    );

    let to_local_atom = interp.intern_atom("to-local");
    interp.dictionary.insert(
        to_local_atom,
        DictEntry {
            value: Value::Builtin(to_local_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Convert datetime to local timezone.\nUsage: datetime to-local => datetime",
            )),
        },
    );

    let datetime_to_string_atom = interp.intern_atom("datetime->string");
    interp.dictionary.insert(
        datetime_to_string_atom,
        DictEntry {
            value: Value::Builtin(datetime_to_string_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Format datetime as ISO 8601 string.\nUsage: datetime datetime->string => string",
            )),
        },
    );

    let string_to_datetime_atom = interp.intern_atom("string->datetime");
    interp.dictionary.insert(
        string_to_datetime_atom,
        DictEntry {
            value: Value::Builtin(string_to_datetime_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Parse ISO 8601 string to datetime.\nUsage: string string->datetime => datetime",
            )),
        },
    );

    let date_less_than_atom = interp.intern_atom("date<");
    interp.dictionary.insert(
        date_less_than_atom,
        DictEntry {
            value: Value::Builtin(date_less_than_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Compare datetimes (less than).\nUsage: datetime1 datetime2 date< => boolean",
            )),
        },
    );

    let date_greater_than_atom = interp.intern_atom("date>");
    interp.dictionary.insert(
        date_greater_than_atom,
        DictEntry {
            value: Value::Builtin(date_greater_than_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Compare datetimes (greater than).\nUsage: datetime1 datetime2 date> => boolean",
            )),
        },
    );

    let date_equal_atom = interp.intern_atom("date=");
    interp.dictionary.insert(
        date_equal_atom,
        DictEntry {
            value: Value::Builtin(date_equal_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Compare datetimes (equal).\nUsage: datetime1 datetime2 date= => boolean",
            )),
        },
    );

    // Duration operations
    let duration_atom = interp.intern_atom("duration");
    interp.dictionary.insert(
        duration_atom,
        DictEntry {
            value: Value::Builtin(duration_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Create duration from components.\nUsage: days hours minutes seconds duration => duration\nExample: 1 2 30 0 duration",
            )),
        },
    );

    let duration_to_seconds_atom = interp.intern_atom("duration->seconds");
    interp.dictionary.insert(
        duration_to_seconds_atom,
        DictEntry {
            value: Value::Builtin(duration_to_seconds_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Convert duration to total seconds.\nUsage: duration duration->seconds => seconds",
            )),
        },
    );

    let date_add_atom = interp.intern_atom("date+");
    interp.dictionary.insert(
        date_add_atom,
        DictEntry {
            value: Value::Builtin(date_add_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Add duration to datetime.\nUsage: datetime duration date+ => datetime",
            )),
        },
    );

    let date_sub_atom = interp.intern_atom("date-");
    interp.dictionary.insert(
        date_sub_atom,
        DictEntry {
            value: Value::Builtin(date_sub_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Subtract datetime or duration.\nUsage: datetime1 datetime2 date- => duration (time between)\nUsage: datetime duration date- => datetime (subtract duration)",
            )),
        },
    );

    let duration_less_than_atom = interp.intern_atom("duration<");
    interp.dictionary.insert(
        duration_less_than_atom,
        DictEntry {
            value: Value::Builtin(duration_less_than_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Compare durations (less than).\nUsage: duration1 duration2 duration< => boolean",
            )),
        },
    );

    let duration_greater_than_atom = interp.intern_atom("duration>");
    interp.dictionary.insert(
        duration_greater_than_atom,
        DictEntry {
            value: Value::Builtin(duration_greater_than_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Compare durations (greater than).\nUsage: duration1 duration2 duration> => boolean",
            )),
        },
    );

    let duration_equal_atom = interp.intern_atom("duration=");
    interp.dictionary.insert(
        duration_equal_atom,
        DictEntry {
            value: Value::Builtin(duration_equal_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Compare durations (equal).\nUsage: duration1 duration2 duration= => boolean",
            )),
        },
    );
    } // end datetime feature block
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::DictEntry;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_all_builtins_registered() {
        let mut interp = setup_interpreter();

        let expected_builtins = [
            // Basic arithmetic
            "+", "-", "*", "/", "//", "div", "mod", "=", // Comparison operations
            "<", ">", "<=", ">=", "!=", // Basic math functions
            "abs", "min", "max", "sqrt", // Advanced math functions
            "pow", "floor", "ceil", "round", // Trigonometric functions
            "sin", "cos", "tan", // Logarithmic functions
            "log", "exp", // Bitwise operations
            "bit-and", "bit-or", "bit-xor", "bit-not", // Shift operations
            "shl", "shr", // Stack operations
            "roll", "pick", "drop", // Return stack operations
            ">r", "r>", "r@", // Control flow & meta
            "def", "val", // exec and if are now special in evaluator
            // I/O operations
            ".",       // String operations
            "->string", // List operations
            "head", "tail", "cons", "list", // Predicates
            "truthy?",
        ];

        for builtin_name in expected_builtins.iter() {
            let atom = interp.intern_atom(builtin_name);
            assert!(
                interp.dictionary.contains_key(&atom),
                "Expected builtin '{}' to be registered",
                builtin_name
            );

            // Verify it's actually a builtin function
            assert!(matches!(
                interp.dictionary.get(&atom),
                Some(DictEntry {
                    value: Value::Builtin(_),
                    ..
                })
            ));
        }
    }
}
