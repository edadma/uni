use crate::interpreter::Interpreter;
use crate::value::Value;

use crate::primitives::{
    add_builtin, sub_builtin, mul_builtin, div_builtin, mod_builtin, eq_builtin,
    drop_builtin, eval_builtin, cons_builtin, list_builtin, head_builtin,
    roll_builtin, pick_builtin, tail_builtin, truthy_predicate_builtin, null_predicate_builtin,
    def_builtin, val_builtin, if_builtin, print_builtin
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
            "+", "-", "*", "/", "mod", "=",
            "roll", "pick", "drop",
            "eval", "if",
            "def", "val",
            "pr",
            "head", "tail", "cons", "list",
            "null?", "truthy?"
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