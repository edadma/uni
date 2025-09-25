// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;
use crate::evaluator::{execute, execute_list};

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

    // RUST CONCEPT: Execute the chosen branch
    // For lists, we need special handling to execute them as code
    // For non-lists, we can execute directly
    match branch_to_execute {
        Value::Pair(_, _) | Value::Nil => {
            // Execute as a list (sequence of operations)
            execute_list(&branch_to_execute, interp)
        },
        _ => {
            // Execute single value directly
            execute(&branch_to_execute, interp)
        }
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
    fn test_if_builtin_true_condition() {
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
    }

    #[test]
    fn test_if_builtin_false_condition() {
        let mut interp = setup_interpreter();

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
    }

    #[test]
    fn test_if_builtin_boolean_conditions() {
        let mut interp = setup_interpreter();

        // Test explicit true boolean
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        interp.push(Value::Boolean(true));
        interp.push(true_branch);
        interp.push(false_branch);
        if_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));

        // Test explicit false boolean
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        interp.push(Value::Boolean(false));
        interp.push(true_branch);
        interp.push(false_branch);
        if_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 99.0));
    }

    #[test]
    fn test_if_builtin_string_truthiness() {
        let mut interp = setup_interpreter();

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

        // Test non-empty string is truthy
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        interp.push(Value::String("hello".into())); // true condition (non-empty string)
        interp.push(true_branch);
        interp.push(false_branch);
        if_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_if_builtin_nil_truthiness() {
        let mut interp = setup_interpreter();

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
    fn test_if_builtin_null_truthiness() {
        let mut interp = setup_interpreter();

        // Test null is falsy
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        interp.push(Value::Null);           // false condition (null is falsy)
        interp.push(true_branch);
        interp.push(false_branch);
        if_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 99.0));
    }

    #[test]
    fn test_if_builtin_atom_truthiness() {
        let mut interp = setup_interpreter();

        // Test atom is truthy
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        let test_atom = interp.intern_atom("test");
        interp.push(Value::Atom(test_atom));
        interp.push(true_branch);
        interp.push(false_branch);
        if_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));

        // Test quoted atom is truthy
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        let quoted_atom = interp.intern_atom("quoted");
        interp.push(Value::QuotedAtom(quoted_atom));
        interp.push(true_branch);
        interp.push(false_branch);
        if_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_if_builtin_list_truthiness() {
        let mut interp = setup_interpreter();

        // Test non-empty list is truthy
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        let non_empty_list = interp.make_list(vec![Value::Number(1.0)]);
        interp.push(non_empty_list);
        interp.push(true_branch);
        interp.push(false_branch);
        if_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_if_builtin_single_value_branches() {
        let mut interp = setup_interpreter();

        // Test with single values as branches (not lists)
        interp.push(Value::Boolean(true));   // condition
        interp.push(Value::Number(42.0));    // true branch (single value)
        interp.push(Value::Number(99.0));    // false branch (single value)
        if_builtin(&mut interp).unwrap();

        // Should execute true branch and push 42
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_if_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = if_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(1.0));
        let result = if_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only two elements
        interp.push(Value::Number(2.0));
        let result = if_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_if_builtin_numeric_edge_cases() {
        let mut interp = setup_interpreter();

        // Test negative number is truthy
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        interp.push(Value::Number(-5.0));   // truthy (non-zero)
        interp.push(true_branch);
        interp.push(false_branch);
        if_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));

        // Test very small number is truthy
        let true_branch = interp.make_list(vec![Value::Number(42.0)]);
        let false_branch = interp.make_list(vec![Value::Number(99.0)]);

        interp.push(Value::Number(0.001));  // truthy (non-zero)
        interp.push(true_branch);
        interp.push(false_branch);
        if_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }
}