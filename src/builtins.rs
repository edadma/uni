use crate::interpreter::Interpreter;
use crate::value::{Value, RuntimeError};

// RUST CONCEPT: Import modular primitives
// Import completed primitives from the new modular structure
use crate::primitives::{
    add_builtin, sub_builtin, mul_builtin, div_builtin, mod_builtin, eq_builtin,
    drop_builtin, eval_builtin, cons_builtin, list_builtin, head_builtin,
    truthy_predicate_builtin
};

// RUST CONCEPT: Builtin function implementations
// Modularized primitives are imported, remaining builtins implemented here


// RUST CONCEPT: The def builtin - defines new words in the dictionary
// Usage: 'word-name definition def
// Examples:
//   'square [dup *] def     - defines a procedure
//   'pi 3.14159 def         - defines a constant
pub fn def_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Stack-based parameter passing
    // def expects two values on the stack:
    // 1. The definition (top of stack) - can be any Value
    // 2. The word name (second on stack) - must be an Atom

    let definition = interp.pop()?;  // What to define the word as
    let name_value = interp.pop()?;  // Name of the word to define

    // RUST CONCEPT: Pattern matching for type checking
    // The word name must be an Atom (interned string)
    match name_value {
        Value::Atom(atom_name) => {
            // RUST CONCEPT: Creating dict entry with executable flag
            use crate::interpreter::DictEntry;
            let entry = DictEntry {
                value: definition,
                is_executable: true,  // def marks entries as executable
            };
            interp.dictionary.insert(atom_name, entry);
            Ok(())
        },

        // RUST CONCEPT: Descriptive error messages
        // If the name isn't an atom, we can't use it as a dictionary key
        _ => Err(RuntimeError::TypeError(
            "def expects an atom as the word name (use 'word-name definition def)".to_string()
        ))
    }
}

// RUST CONCEPT: The val builtin - defines constants only (like Scheme's define for constants)
// Usage: 'constant-name value val
// Examples:
//   'pi 3.14159 val         - defines a constant
//   'greeting "Hello!" val  - defines a string constant
// Unlike def, val is specifically for constants that shouldn't be evaluated
pub fn val_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Same implementation as def for now
    // The distinction is semantic - val is for constants, def is for procedures
    let definition = interp.pop()?;  // The constant value
    let name_value = interp.pop()?;  // Name of the constant

    match name_value {
        Value::Atom(atom_name) => {
            // RUST CONCEPT: Creating dict entry with constant flag
            use crate::interpreter::DictEntry;
            let entry = DictEntry {
                value: definition,
                is_executable: false,  // val marks entries as constants
            };
            interp.dictionary.insert(atom_name, entry);
            Ok(())
        },
        _ => Err(RuntimeError::TypeError(
            "val expects an atom as the constant name (use 'name value val)".to_string()
        ))
    }
}

// RUST CONCEPT: ANS Forth roll primitive
// roll ( n -- ) - Rotate n+1 stack items
// n=0: no effect, n=1: swap top two, n=2: rot three items, etc.
// Example: 1 2 3 4  2 roll  ->  1 3 4 2 (moved item at depth 2 to top)
pub fn roll_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()? as usize;

    // RUST CONCEPT: Bounds checking
    // We need at least n+1 items on the stack
    if interp.stack.len() < n + 1 {
        return Err(RuntimeError::StackUnderflow);
    }

    if n == 0 {
        // n=0: no operation
        return Ok(());
    }

    // RUST CONCEPT: Vec manipulation
    // Remove the item at position n from the end (0-indexed from top)
    let stack_len = interp.stack.len();
    let item = interp.stack.remove(stack_len - n - 1);

    // Push it to the top
    interp.stack.push(item);

    Ok(())
}

// RUST CONCEPT: ANS Forth pick primitive
// pick ( n -- value ) - Copy the nth item from the stack to the top
// n=0: dup, n=1: over, n=2: pick third item, etc.
// Example: 1 2 3 4  2 pick  ->  1 2 3 4 2 (copied item at depth 2 to top)
pub fn pick_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()? as usize;

    // RUST CONCEPT: Bounds checking
    // We need at least n+1 items on the remaining stack
    if interp.stack.len() < n + 1 {
        return Err(RuntimeError::StackUnderflow);
    }

    // RUST CONCEPT: Vec indexing from the end
    // Get the item at position n from the top (0-indexed)
    let stack_len = interp.stack.len();
    let item = interp.stack[stack_len - n - 1].clone();

    // Push a copy to the top
    interp.stack.push(item);

    Ok(())
}

// RUST CONCEPT: Conditional execution builtin
// if ( condition true-branch false-branch -- ... )
// Pops condition, true-branch, false-branch from stack
// If condition is non-zero/true, evaluates true-branch, otherwise false-branch
// Example: 1 [42 pr] [99 pr] if  -> prints 42
//          0 [42 pr] [99 pr] if  -> prints 99
pub fn if_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {

    // RUST CONCEPT: Stack order - items are popped in reverse order
    let false_branch = interp.pop()?;  // Top of stack
    let true_branch = interp.pop()?;   // Second item
    let condition = interp.pop()?;     // Third item (bottom of the three)

    // RUST CONCEPT: Truthiness evaluation (JavaScript-style)
    // We need to determine if the condition is "true"
    let is_true = match condition {
        Value::Boolean(b) => b,        // Boolean values use their literal truth value
        Value::Number(n) => n != 0.0,  // Zero is false, non-zero is true
        Value::String(s) => !s.is_empty(),  // Empty string is false, non-empty is true
        Value::Null => false,          // Null is falsy
        Value::Nil => true,            // Empty list is truthy (like [] in JS)
        Value::Atom(_) => true,        // Atoms are true
        Value::QuotedAtom(_) => true,  // Quoted atoms are true
        Value::Pair(_, _) => true,     // Non-empty lists are true
        Value::Builtin(_) => true,     // Builtins are true
    };

    // RUST CONCEPT: Conditional execution
    let branch_to_execute = if is_true { true_branch } else { false_branch };

    // RUST CONCEPT: Execute the chosen branch like eval does
    // We can use eval_builtin which handles both lists and single values
    interp.push(branch_to_execute);
    eval_builtin(interp)
}

// RUST CONCEPT: Print builtin - pops and displays the top stack value
// Usage: 42 pr  (prints "42" and removes it from stack)
// Note: We use "pr" instead of "." because "." is reserved for cons pair notation
// RUST CONCEPT: Helper function to print any Value type
// Eliminates code duplication by handling all Value printing in one place
fn print_value(value: &Value) {
    match value {
        Value::Number(n) => print!("{}", n),
        Value::String(s) => print!("\"{}\"", s),
        Value::Boolean(b) => print!("{}", if *b { "true" } else { "false" }),
        Value::Null => print!("null"),
        Value::Atom(atom) => print!("{}", atom),
        Value::QuotedAtom(atom) => print!("'{}", atom),
        Value::Nil => print!("[]"),
        Value::Pair(_, _) => print_list(value),
        Value::Builtin(_) => print!("[builtin]"),
    }
}

fn print_list(value: &Value) {
    print!("[");
    print_list_contents(value, true);
    print!("]");
}

fn print_list_contents(value: &Value, first: bool) {
    match value {
        Value::Nil => {
            // End of list - don't print anything
        },
        Value::Pair(car, cdr) => {
            if !first {
                print!(" ");
            }

            // Print the car (first element) - no duplication!
            print_value(car.as_ref());

            // Recursively print the cdr (rest of list)
            print_list_contents(cdr.as_ref(), false);
        },
        _ => {
            // Improper list (dotted pair) - show the dot notation
            print!(" . ");
            print_value(value);
        }
    }
}

pub fn print_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;

    // RUST CONCEPT: No duplication - use shared print_value function
    print_value(&value);
    println!(); // Add newline after printing

    Ok(())
}

// RUST CONCEPT: List operations - head builtin

// RUST CONCEPT: List operations - tail builtin
// tail ( list -- list ) - Get rest of list after first element
// Example: [1 2 3] tail -> [2 3]
pub fn tail_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let list = interp.pop()?;

    match list {
        Value::Pair(_, cdr) => {
            // Return the rest of the list (cdr)
            interp.push((*cdr).clone());
            Ok(())
        },
        Value::Nil => {
            // Tail of empty list is empty list
            interp.push(Value::Nil);
            Ok(())
        },
        _ => {
            // Not a list
            Err(RuntimeError::TypeError("tail expects a list".to_string()))
        }
    }
}


// RUST CONCEPT: Type checking predicates
// null? ( value -- boolean ) - Check if value is null
pub fn null_predicate_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    let is_null = interp.is_null(&value);
    interp.push(Value::Boolean(is_null));
    Ok(())
}

// truthy? ( value -- boolean ) - Check if value is truthy

// RUST CONCEPT: Registering all builtins with the interpreter
pub fn register_builtins(interp: &mut Interpreter) {
    use crate::interpreter::DictEntry;

    // RUST CONCEPT: Creating atoms for builtin names
    // We intern the names so they can be looked up in the dictionary
    // Builtins are always executable

    // Arithmetic operations
    let add_atom = interp.intern_atom("+");
    interp.dictionary.insert(add_atom, DictEntry {
        value: Value::Builtin(add_builtin),
        is_executable: true,  // builtins are always executable
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

    // Stack operations - primitives
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

    // The def builtin for defining new words
    let def_atom = interp.intern_atom("def");
    interp.dictionary.insert(def_atom, DictEntry {
        value: Value::Builtin(def_builtin),
        is_executable: true,
    });

    // The val builtin for defining constants
    let val_atom = interp.intern_atom("val");
    interp.dictionary.insert(val_atom, DictEntry {
        value: Value::Builtin(val_builtin),
        is_executable: true,
    });

    // The print builtin for output
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
    use crate::value::Value;
    use crate::interpreter::DictEntry;

    fn setup_interpreter() -> Interpreter {
        // RUST CONCEPT: Automatic initialization
        // Interpreter::new() now automatically loads builtins and stdlib
        Interpreter::new()
    }


    #[test]
    fn test_division_by_zero() {
        let mut interp = setup_interpreter();

        interp.push(Value::Number(5.0));
        interp.push(Value::Number(0.0));

        let result = div_builtin(&mut interp);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivisionByZero));
    }


    #[test]
    fn test_mod_by_zero() {
        let mut interp = setup_interpreter();

        interp.push(Value::Number(5.0));
        interp.push(Value::Number(0.0));

        let result = mod_builtin(&mut interp);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::ModuloByZero));
    }



    #[test]
    fn test_roll_builtin() {
        let mut interp = setup_interpreter();

        // Test roll with n=1 (should swap top two)
        // Stack: 1 2 3  ->  1 roll  ->  1 3 2
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(1.0)); // roll argument
        roll_builtin(&mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();
        let third = interp.pop().unwrap();

        assert!(matches!(top, Value::Number(n) if n == 2.0));
        assert!(matches!(second, Value::Number(n) if n == 3.0));
        assert!(matches!(third, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_pick_builtin() {
        let mut interp = setup_interpreter();

        // Test pick with n=1 (should copy second item to top)
        // Stack: 1 2 3  ->  1 pick  ->  1 2 3 2
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(1.0)); // pick argument
        pick_builtin(&mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();
        let third = interp.pop().unwrap();
        let fourth = interp.pop().unwrap();

        assert!(matches!(top, Value::Number(n) if n == 2.0)); // copied item
        assert!(matches!(second, Value::Number(n) if n == 3.0));
        assert!(matches!(third, Value::Number(n) if n == 2.0));
        assert!(matches!(fourth, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_if_builtin() {
        let mut interp = setup_interpreter();

        // Test true condition - if should execute the true branch
        // Create branches that push specific numbers
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);   // [42]
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);  // [99]

        interp.push(Value::Number(1.0));    // true condition
        interp.push(true_branch);           // true branch
        interp.push(false_branch);          // false branch
        if_builtin(&mut interp).unwrap();

        // Should have executed the true branch, which pushes 42
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));

        // Test false condition - if should execute the false branch
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        interp.push(Value::Number(0.0));    // false condition
        interp.push(true_branch);           // true branch
        interp.push(false_branch);          // false branch
        if_builtin(&mut interp).unwrap();

        // Should have executed the false branch, which pushes 99
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 99.0));

        // Test empty string is falsy
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        interp.push(Value::String("".into()));  // false condition (empty string)
        interp.push(true_branch);               // true branch
        interp.push(false_branch);              // false branch
        if_builtin(&mut interp).unwrap();

        // Should execute false branch because empty string is falsy
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 99.0));

        // Test empty list is truthy (like [] in JavaScript)
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        interp.push(Value::Nil);             // true condition (empty list is truthy)
        interp.push(true_branch);            // true branch
        interp.push(false_branch);           // false branch
        if_builtin(&mut interp).unwrap();

        // Should execute true branch because empty list is truthy
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }


    #[test]
    fn test_all_builtins_registered() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing that all expected builtins are in the dictionary
        let expected_builtins = ["+", "-", "*", "/", "mod", "=", "roll", "pick", "drop", "eval", "if", "def", "val", "pr", "head", "tail"];

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

    #[test]
    fn test_def_builtin_constant() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing constant definition
        // Define pi as 3.14159: 'pi 3.14159 def
        let pi_atom = interp.intern_atom("pi");
        interp.push(Value::Atom(pi_atom));      // Word name
        interp.push(Value::Number(3.14159));    // Definition
        def_builtin(&mut interp).unwrap();

        // Verify it was stored in dictionary
        let pi_lookup = interp.intern_atom("pi");
        assert!(interp.dictionary.contains_key(&pi_lookup));

        // Verify we can retrieve the constant
        match interp.dictionary.get(&pi_lookup) {
            Some(DictEntry { value: Value::Number(n), .. }) => assert!((n - 3.14159).abs() < 1e-10),
            _ => panic!("Expected pi to be defined as a number"),
        }

        // Clear stack before next test
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_procedure() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing procedure definition
        // Define square as [dup *]: 'square [dup *] def
        let square_atom = interp.intern_atom("square");
        let dup_atom = interp.intern_atom("dup");
        let mul_atom = interp.intern_atom("*");

        // Create the procedure list [dup *]
        let square_proc = interp.make_list(vec![
            Value::Atom(dup_atom),
            Value::Atom(mul_atom),
        ]);

        interp.push(Value::Atom(square_atom));  // Word name
        interp.push(square_proc);               // Definition
        def_builtin(&mut interp).unwrap();

        // Verify it was stored in dictionary
        let square_lookup = interp.intern_atom("square");
        assert!(interp.dictionary.contains_key(&square_lookup));

        // Verify we can retrieve the procedure
        match interp.dictionary.get(&square_lookup) {
            Some(DictEntry { value: Value::Pair(_, _), .. }) => (), // It's a list (procedure)
            _ => panic!("Expected square to be defined as a list"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_string_definition() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing string definition
        // Define greeting as "Hello, Uni!": 'greeting "Hello, Uni!" def
        let greeting_atom = interp.intern_atom("greeting");
        let greeting_string: std::rc::Rc<str> = "Hello, Uni!".into();

        interp.push(Value::Atom(greeting_atom));           // Word name
        interp.push(Value::String(greeting_string));       // Definition
        def_builtin(&mut interp).unwrap();

        // Verify it was stored
        let greeting_lookup = interp.intern_atom("greeting");
        match interp.dictionary.get(&greeting_lookup) {
            Some(DictEntry { value: Value::String(s), .. }) => assert_eq!(s.as_ref(), "Hello, Uni!"),
            _ => panic!("Expected greeting to be defined as a string"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_error_cases() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing error handling
        // def requires exactly two arguments
        assert!(def_builtin(&mut interp).is_err()); // Empty stack

        interp.push(Value::Number(42.0));
        assert!(def_builtin(&mut interp).is_err()); // Only one argument

        // RUST CONCEPT: Testing type safety
        // First argument (word name) must be an Atom
        interp.push(Value::Number(42.0));        // Invalid name (not atom)
        interp.push(Value::Number(123.0));       // Definition
        let result = def_builtin(&mut interp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("def expects an atom"));

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_redefinition() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing word redefinition
        // First define foo as 123
        let foo_atom = interp.intern_atom("foo");
        interp.push(Value::Atom(foo_atom.clone()));
        interp.push(Value::Number(123.0));
        def_builtin(&mut interp).unwrap();

        // Verify first definition
        match interp.dictionary.get(&foo_atom) {
            Some(DictEntry { value: Value::Number(n), .. }) => assert_eq!(*n, 123.0),
            _ => panic!("Expected foo to be 123"),
        }

        // Redefine foo as 456
        interp.push(Value::Atom(foo_atom.clone()));
        interp.push(Value::Number(456.0));
        def_builtin(&mut interp).unwrap();

        // Verify redefinition worked
        match interp.dictionary.get(&foo_atom) {
            Some(DictEntry { value: Value::Number(n), .. }) => assert_eq!(*n, 456.0),
            _ => panic!("Expected foo to be redefined as 456"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_with_nil() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing edge case - defining with empty list
        let empty_atom = interp.intern_atom("empty");
        interp.push(Value::Atom(empty_atom.clone()));
        interp.push(Value::Nil);
        def_builtin(&mut interp).unwrap();

        // Verify nil definition
        match interp.dictionary.get(&empty_atom) {
            Some(DictEntry { value: Value::Nil, .. }) => (),
            _ => panic!("Expected empty to be defined as Nil"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_val_builtin_constants() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing val for defining constants
        // Define pi: 'pi 3.14159 val
        let pi_atom = interp.intern_atom("pi");
        interp.push(Value::Atom(pi_atom.clone()));
        interp.push(Value::Number(3.14159));
        val_builtin(&mut interp).unwrap();

        // Verify it was stored
        match interp.dictionary.get(&pi_atom) {
            Some(DictEntry { value: Value::Number(n), .. }) => assert!((n - 3.14159).abs() < 1e-10),
            _ => panic!("Expected pi constant"),
        }

        // Define string constant: 'greeting "Hello!" val
        let greeting_atom = interp.intern_atom("greeting");
        let hello_str: std::rc::Rc<str> = "Hello!".into();
        interp.push(Value::Atom(greeting_atom.clone()));
        interp.push(Value::String(hello_str));
        val_builtin(&mut interp).unwrap();

        // Verify string constant
        match interp.dictionary.get(&greeting_atom) {
            Some(DictEntry { value: Value::String(s), .. }) => assert_eq!(s.as_ref(), "Hello!"),
            _ => panic!("Expected greeting string constant"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_val_builtin_error_cases() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing val error handling
        // Same error conditions as def
        assert!(val_builtin(&mut interp).is_err()); // Empty stack

        interp.push(Value::Number(42.0));
        assert!(val_builtin(&mut interp).is_err()); // Only one argument

        // Non-atom name should fail
        interp.push(Value::Number(42.0));        // Invalid name
        interp.push(Value::Number(123.0));       // Value
        let result = val_builtin(&mut interp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("val expects an atom"));

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_vs_val_semantic_distinction() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing semantic difference between def and val
        // Both should work the same way at the builtin level
        // The distinction is in usage: def for procedures, val for constants

        // Use val for a number constant
        let num_atom = interp.intern_atom("mynum");
        interp.push(Value::Atom(num_atom.clone()));
        interp.push(Value::Number(42.0));
        val_builtin(&mut interp).unwrap();

        // Use def for a procedure (list)
        let proc_atom = interp.intern_atom("myproc");
        let dup_atom = interp.intern_atom("dup");
        let add_atom = interp.intern_atom("+");
        let proc_list = interp.make_list(vec![
            Value::Atom(dup_atom),
            Value::Atom(add_atom),
        ]);
        interp.push(Value::Atom(proc_atom.clone()));
        interp.push(proc_list);
        def_builtin(&mut interp).unwrap();

        // Both should be in dictionary
        assert!(interp.dictionary.contains_key(&num_atom));
        assert!(interp.dictionary.contains_key(&proc_atom));

        // Verify types
        assert!(matches!(interp.dictionary.get(&num_atom), Some(DictEntry { value: Value::Number(_), .. })));
        assert!(matches!(interp.dictionary.get(&proc_atom), Some(DictEntry { value: Value::Pair(_, _), .. })));

        // Clear stack
        while interp.pop().is_ok() {}
    }


    #[test]
    fn test_tail_builtin() {
        let mut interp = setup_interpreter();

        // Test tail of [1 2 3] -> [2 3]
        let list = interp.make_list(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0)
        ]);
        interp.push(list);
        tail_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // Should be a list [2 3]
        assert!(matches!(result, Value::Pair(_, _)));

        // Test tail of single element list [42] -> []
        let single = interp.make_list(vec![Value::Number(42.0)]);
        interp.push(single);
        tail_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Nil));

        // Test tail of empty list -> []
        interp.push(Value::Nil);
        tail_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Nil));

        // Test tail of non-list should error
        interp.push(Value::Number(42.0));
        let result = tail_builtin(&mut interp);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::TypeError(msg) if msg.contains("tail expects a list")));
    }




    #[test]
    fn test_null_predicate_builtin() {
        let mut interp = setup_interpreter();

        // Test null? with null value -> true
        interp.push(Value::Null);
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test null? with non-null values -> false
        let test_cases = vec![
            Value::Boolean(false),
            Value::Boolean(true),
            Value::Number(0.0),
            Value::Number(42.0),
            Value::String("".into()),
            Value::String("hello".into()),
            Value::Nil,
            Value::Atom(interp.intern_atom("test")),
        ];

        for test_value in test_cases {
            interp.push(test_value);
            null_predicate_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Boolean(false)),
                "Expected false for non-null value");
        }

        // Test stack underflow
        let result = null_predicate_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }



}

