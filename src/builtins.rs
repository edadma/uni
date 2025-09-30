use crate::interpreter::Interpreter;
use crate::value::Value;
use std::rc::Rc;

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
    ceil_builtin,
    // List operations
    cons_builtin,
    cos_builtin,
    // Meta operations
    def_builtin,
    div_builtin,
    doc_builtin,
    // Stack operations
    drop_builtin,
    eq_builtin,
    exp_builtin,
    floor_builtin,
    from_r_builtin,
    greater_equal_builtin,
    greater_than_builtin,
    head_builtin,
    help_builtin,
    less_equal_builtin,
    // Comparison operations
    less_than_builtin,
    list_builtin,
    list_to_vector_builtin,
    // Logarithmic functions
    log_builtin,
    make_vector_builtin,
    max_builtin,
    min_builtin,
    mod_builtin,
    mul_builtin,
    not_equal_builtin,
    null_predicate_builtin,
    pick_builtin,
    // Advanced math functions
    pow_builtin,
    // Control flow - if is now special in evaluator
    // I/O operations
    print_builtin,
    r_fetch_builtin,
    roll_builtin,
    round_builtin,
    // Shift operations
    shl_builtin,
    shr_builtin,
    // Trigonometric functions
    sin_builtin,
    sqrt_builtin,
    sub_builtin,
    tail_builtin,
    tan_builtin,
    // Return stack operations
    to_r_builtin,
    // String operations
    to_string_builtin,
    // Predicate operations
    truthy_predicate_builtin,
    val_builtin,
    vector_builtin,
    vector_length_builtin,
    vector_ref_builtin,
    vector_set_builtin,
    vector_to_list_builtin,
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
    let print_atom = interp.intern_atom("pr");
    interp.dictionary.insert(
        print_atom,
        DictEntry {
            value: Value::Builtin(print_builtin),
            is_executable: true,
            doc: Some(Rc::<str>::from(
                "Print top stack value to output.\nUsage: value pr => (prints value)\nExample: \"Hello\" pr => Hello",
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
                "Set element at index in vector.\nUsage: #[a b c] i value vector-set! => #[a value c]\nExample: #[10 20 30] 1 99 vector-set! => #[10 99 30]",
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

    // Advanced math functions
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

    // Trigonometric functions
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

    // Logarithmic functions
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
            "+", "-", "*", "/", "mod", "=", // Comparison operations
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
            "pr",       // String operations
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
