use crate::interpreter::Interpreter;
use crate::value::Value;

use crate::primitives::{
    // Basic arithmetic
    add_builtin, sub_builtin, mul_builtin, div_builtin, mod_builtin, eq_builtin,
    // Comparison operations
    less_than_builtin, greater_than_builtin, less_equal_builtin, greater_equal_builtin, not_equal_builtin,
    // Basic math functions
    abs_builtin, min_builtin, max_builtin, sqrt_builtin,
    // Advanced math functions
    pow_builtin, floor_builtin, ceil_builtin, round_builtin,
    // Trigonometric functions
    sin_builtin, cos_builtin, tan_builtin,
    // Logarithmic functions
    log_builtin, exp_builtin,
    // Bitwise operations
    bit_and_builtin, bit_or_builtin, bit_xor_builtin, bit_not_builtin,
    // Shift operations
    shl_builtin, shr_builtin,
    // Stack operations
    drop_builtin, eval_builtin, roll_builtin, pick_builtin,
    // List operations
    cons_builtin, list_builtin, head_builtin, tail_builtin,
    // Meta operations
    def_builtin, val_builtin,
    // Control flow
    if_builtin,
    // I/O operations
    print_builtin,
    // Predicate operations
    truthy_predicate_builtin, null_predicate_builtin
};

// RUST CONCEPT: Registering all builtins with the interpreter
pub fn register_builtins(interp: &mut Interpreter) {
    use crate::interpreter::DictEntry;

    // Arithmetic operations
    let add_atom = interp.intern_atom("+");
    interp.dictionary.insert(add_atom, DictEntry {
        value: Value::Builtin(add_builtin),
        is_executable: true,
    });

    let sub_atom = interp.intern_atom("-");
    interp.dictionary.insert(sub_atom, DictEntry {
        value: Value::Builtin(sub_builtin),
        is_executable: true,
    });

    let mul_atom = interp.intern_atom("*");
    interp.dictionary.insert(mul_atom, DictEntry {
        value: Value::Builtin(mul_builtin),
        is_executable: true,
    });

    let div_atom = interp.intern_atom("/");
    interp.dictionary.insert(div_atom, DictEntry {
        value: Value::Builtin(div_builtin),
        is_executable: true,
    });

    let mod_atom = interp.intern_atom("mod");
    interp.dictionary.insert(mod_atom, DictEntry {
        value: Value::Builtin(mod_builtin),
        is_executable: true,
    });

    let eq_atom = interp.intern_atom("=");
    interp.dictionary.insert(eq_atom, DictEntry {
        value: Value::Builtin(eq_builtin),
        is_executable: true,
    });

    // Stack operations
    let roll_atom = interp.intern_atom("roll");
    interp.dictionary.insert(roll_atom, DictEntry {
        value: Value::Builtin(roll_builtin),
        is_executable: true,
    });

    let pick_atom = interp.intern_atom("pick");
    interp.dictionary.insert(pick_atom, DictEntry {
        value: Value::Builtin(pick_builtin),
        is_executable: true,
    });

    let drop_atom = interp.intern_atom("drop");
    interp.dictionary.insert(drop_atom, DictEntry {
        value: Value::Builtin(drop_builtin),
        is_executable: true,
    });

    // The crucial eval builtin
    let eval_atom = interp.intern_atom("eval");
    interp.dictionary.insert(eval_atom, DictEntry {
        value: Value::Builtin(eval_builtin),
        is_executable: true,
    });

    // Control flow
    let if_atom = interp.intern_atom("if");
    interp.dictionary.insert(if_atom, DictEntry {
        value: Value::Builtin(if_builtin),
        is_executable: true,
    });

    // Dictionary operations
    let def_atom = interp.intern_atom("def");
    interp.dictionary.insert(def_atom, DictEntry {
        value: Value::Builtin(def_builtin),
        is_executable: true,
    });

    let val_atom = interp.intern_atom("val");
    interp.dictionary.insert(val_atom, DictEntry {
        value: Value::Builtin(val_builtin),
        is_executable: true,
    });

    // I/O operations
    let print_atom = interp.intern_atom("pr");
    interp.dictionary.insert(print_atom, DictEntry {
        value: Value::Builtin(print_builtin),
        is_executable: true,
    });

    // List operations
    let head_atom = interp.intern_atom("head");
    interp.dictionary.insert(head_atom, DictEntry {
        value: Value::Builtin(head_builtin),
        is_executable: true,
    });

    let tail_atom = interp.intern_atom("tail");
    interp.dictionary.insert(tail_atom, DictEntry {
        value: Value::Builtin(tail_builtin),
        is_executable: true,
    });

    let cons_atom = interp.intern_atom("cons");
    interp.dictionary.insert(cons_atom, DictEntry {
        value: Value::Builtin(cons_builtin),
        is_executable: true,
    });

    let list_atom = interp.intern_atom("list");
    interp.dictionary.insert(list_atom, DictEntry {
        value: Value::Builtin(list_builtin),
        is_executable: true,
    });

    // Type checking predicates
    let null_predicate_atom = interp.intern_atom("null?");
    interp.dictionary.insert(null_predicate_atom, DictEntry {
        value: Value::Builtin(null_predicate_builtin),
        is_executable: true,
    });

    let truthy_predicate_atom = interp.intern_atom("truthy?");
    interp.dictionary.insert(truthy_predicate_atom, DictEntry {
        value: Value::Builtin(truthy_predicate_builtin),
        is_executable: true,
    });

    // Comparison operations
    let less_than_atom = interp.intern_atom("<");
    interp.dictionary.insert(less_than_atom, DictEntry {
        value: Value::Builtin(less_than_builtin),
        is_executable: true,
    });

    let greater_than_atom = interp.intern_atom(">");
    interp.dictionary.insert(greater_than_atom, DictEntry {
        value: Value::Builtin(greater_than_builtin),
        is_executable: true,
    });

    let less_equal_atom = interp.intern_atom("<=");
    interp.dictionary.insert(less_equal_atom, DictEntry {
        value: Value::Builtin(less_equal_builtin),
        is_executable: true,
    });

    let greater_equal_atom = interp.intern_atom(">=");
    interp.dictionary.insert(greater_equal_atom, DictEntry {
        value: Value::Builtin(greater_equal_builtin),
        is_executable: true,
    });

    let not_equal_atom = interp.intern_atom("!=");
    interp.dictionary.insert(not_equal_atom, DictEntry {
        value: Value::Builtin(not_equal_builtin),
        is_executable: true,
    });

    // Basic math functions
    let abs_atom = interp.intern_atom("abs");
    interp.dictionary.insert(abs_atom, DictEntry {
        value: Value::Builtin(abs_builtin),
        is_executable: true,
    });

    let min_atom = interp.intern_atom("min");
    interp.dictionary.insert(min_atom, DictEntry {
        value: Value::Builtin(min_builtin),
        is_executable: true,
    });

    let max_atom = interp.intern_atom("max");
    interp.dictionary.insert(max_atom, DictEntry {
        value: Value::Builtin(max_builtin),
        is_executable: true,
    });

    let sqrt_atom = interp.intern_atom("sqrt");
    interp.dictionary.insert(sqrt_atom, DictEntry {
        value: Value::Builtin(sqrt_builtin),
        is_executable: true,
    });

    // Advanced math functions
    let pow_atom = interp.intern_atom("pow");
    interp.dictionary.insert(pow_atom, DictEntry {
        value: Value::Builtin(pow_builtin),
        is_executable: true,
    });

    let floor_atom = interp.intern_atom("floor");
    interp.dictionary.insert(floor_atom, DictEntry {
        value: Value::Builtin(floor_builtin),
        is_executable: true,
    });

    let ceil_atom = interp.intern_atom("ceil");
    interp.dictionary.insert(ceil_atom, DictEntry {
        value: Value::Builtin(ceil_builtin),
        is_executable: true,
    });

    let round_atom = interp.intern_atom("round");
    interp.dictionary.insert(round_atom, DictEntry {
        value: Value::Builtin(round_builtin),
        is_executable: true,
    });

    // Trigonometric functions
    let sin_atom = interp.intern_atom("sin");
    interp.dictionary.insert(sin_atom, DictEntry {
        value: Value::Builtin(sin_builtin),
        is_executable: true,
    });

    let cos_atom = interp.intern_atom("cos");
    interp.dictionary.insert(cos_atom, DictEntry {
        value: Value::Builtin(cos_builtin),
        is_executable: true,
    });

    let tan_atom = interp.intern_atom("tan");
    interp.dictionary.insert(tan_atom, DictEntry {
        value: Value::Builtin(tan_builtin),
        is_executable: true,
    });

    // Logarithmic functions
    let log_atom = interp.intern_atom("log");
    interp.dictionary.insert(log_atom, DictEntry {
        value: Value::Builtin(log_builtin),
        is_executable: true,
    });

    let exp_atom = interp.intern_atom("exp");
    interp.dictionary.insert(exp_atom, DictEntry {
        value: Value::Builtin(exp_builtin),
        is_executable: true,
    });

    // Bitwise operations
    let bit_and_atom = interp.intern_atom("bit-and");
    interp.dictionary.insert(bit_and_atom, DictEntry {
        value: Value::Builtin(bit_and_builtin),
        is_executable: true,
    });

    let bit_or_atom = interp.intern_atom("bit-or");
    interp.dictionary.insert(bit_or_atom, DictEntry {
        value: Value::Builtin(bit_or_builtin),
        is_executable: true,
    });

    let bit_xor_atom = interp.intern_atom("bit-xor");
    interp.dictionary.insert(bit_xor_atom, DictEntry {
        value: Value::Builtin(bit_xor_builtin),
        is_executable: true,
    });

    let bit_not_atom = interp.intern_atom("bit-not");
    interp.dictionary.insert(bit_not_atom, DictEntry {
        value: Value::Builtin(bit_not_builtin),
        is_executable: true,
    });

    // Shift operations
    let shl_atom = interp.intern_atom("shl");
    interp.dictionary.insert(shl_atom, DictEntry {
        value: Value::Builtin(shl_builtin),
        is_executable: true,
    });

    let shr_atom = interp.intern_atom("shr");
    interp.dictionary.insert(shr_atom, DictEntry {
        value: Value::Builtin(shr_builtin),
        is_executable: true,
    });
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
            "+", "-", "*", "/", "mod", "=",
            // Comparison operations
            "<", ">", "<=", ">=", "!=",
            // Basic math functions
            "abs", "min", "max", "sqrt",
            // Advanced math functions
            "pow", "floor", "ceil", "round",
            // Trigonometric functions
            "sin", "cos", "tan",
            // Logarithmic functions
            "log", "exp",
            // Bitwise operations
            "bit-and", "bit-or", "bit-xor", "bit-not",
            // Shift operations
            "shl", "shr",
            // Stack operations
            "roll", "pick", "drop",
            // Control flow & meta
            "eval", "if", "def", "val",
            // I/O operations
            "pr",
            // List operations
            "head", "tail", "cons", "list",
            // Predicates
            "truthy?"
        ];

        for builtin_name in expected_builtins.iter() {
            let atom = interp.intern_atom(builtin_name);
            assert!(interp.dictionary.contains_key(&atom),
                   "Expected builtin '{}' to be registered", builtin_name);

            // Verify it's actually a builtin function
            assert!(matches!(
                interp.dictionary.get(&atom),
                Some(DictEntry { value: Value::Builtin(_), .. })
            ));
        }
    }
}