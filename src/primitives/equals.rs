// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use std::rc::Rc;

// RUST CONCEPT: Equality comparison across all value types
// Stack-based equality: ( value1 value2 -- boolean )
pub fn eq_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    let result = match (&a, &b) {
        (Value::Number(n1), Value::Number(n2)) => n1 == n2,
        (Value::String(s1), Value::String(s2)) => s1 == s2,
        (Value::Atom(a1), Value::Atom(a2)) => a1 == a2,
        (Value::QuotedAtom(a1), Value::QuotedAtom(a2)) => a1 == a2,
        (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
        (Value::Null, Value::Null) => true,
        (Value::Nil, Value::Nil) => true,
        // Lists (Pairs) require deep comparison
        (Value::Pair(car1, cdr1), Value::Pair(car2, cdr2)) => {
            // Recursive equality check for cons cells
            eq_values(car1, car2) && eq_values(cdr1, cdr2)
        }
        // Different types are never equal
        _ => false,
    };

    interp.push(Value::Boolean(result));
    Ok(())
}

// RUST CONCEPT: Recursive helper function for deep equality
// This avoids stack overflow from the interpreter's stack operations
fn eq_values(a: &Rc<Value>, b: &Rc<Value>) -> bool {
    match (a.as_ref(), b.as_ref()) {
        (Value::Number(n1), Value::Number(n2)) => n1 == n2,
        (Value::String(s1), Value::String(s2)) => s1 == s2,
        (Value::Atom(a1), Value::Atom(a2)) => a1 == a2,
        (Value::QuotedAtom(a1), Value::QuotedAtom(a2)) => a1 == a2,
        (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
        (Value::Null, Value::Null) => true,
        (Value::Nil, Value::Nil) => true,
        (Value::Pair(car1, cdr1), Value::Pair(car2, cdr2)) => {
            eq_values(car1, car2) && eq_values(cdr1, cdr2)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_eq_builtin_numbers() {
        let mut interp = setup_interpreter();

        // Test equal numbers
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(42.0));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test unequal numbers
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(5.0));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_eq_builtin_strings() {
        let mut interp = setup_interpreter();

        // Test equal strings
        interp.push(Value::String("hello".into()));
        interp.push(Value::String("hello".into()));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test unequal strings
        interp.push(Value::String("hello".into()));
        interp.push(Value::String("world".into()));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_eq_builtin_booleans() {
        let mut interp = setup_interpreter();

        // Test equal booleans
        interp.push(Value::Boolean(true));
        interp.push(Value::Boolean(true));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test unequal booleans
        interp.push(Value::Boolean(true));
        interp.push(Value::Boolean(false));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_eq_builtin_null() {
        let mut interp = setup_interpreter();

        // Test null vs null
        interp.push(Value::Null);
        interp.push(Value::Null);
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test null vs other types
        interp.push(Value::Null);
        interp.push(Value::Boolean(false));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_eq_builtin_different_types() {
        let mut interp = setup_interpreter();

        // Numbers and strings should never be equal
        interp.push(Value::Number(42.0));
        interp.push(Value::String("42".into()));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // Boolean true and number 1 should never be equal
        interp.push(Value::Boolean(true));
        interp.push(Value::Number(1.0));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_eq_builtin_lists() {
        let mut interp = setup_interpreter();

        // Create identical lists [1, 2]
        let list1 = interp.make_list(vec![Value::Number(1.0), Value::Number(2.0)]);
        let list2 = interp.make_list(vec![Value::Number(1.0), Value::Number(2.0)]);

        interp.push(list1);
        interp.push(list2);
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Create different lists [1, 2] vs [1, 3]
        let list3 = interp.make_list(vec![Value::Number(1.0), Value::Number(2.0)]);
        let list4 = interp.make_list(vec![Value::Number(1.0), Value::Number(3.0)]);

        interp.push(list3);
        interp.push(list4);
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_eq_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = eq_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = eq_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_eq_builtin_edge_cases_boolean_null() {
        let mut interp = setup_interpreter();

        // Edge case: null vs boolean false (should be false - different types)
        interp.push(Value::Null);
        interp.push(Value::Boolean(false));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // Edge case: null vs number 0 (should be false - different types)
        interp.push(Value::Null);
        interp.push(Value::Number(0.0));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // Edge case: boolean true vs number 1 (should be false - different types)
        interp.push(Value::Boolean(true));
        interp.push(Value::Number(1.0));
        eq_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }
}
