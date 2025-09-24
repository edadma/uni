// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

// RUST CONCEPT: Code evaluation - executing lists as code
// Stack-based eval: ( code -- ... )
pub fn eval_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let code = interp.pop()?;
    execute_list(&code, interp)
}

// RUST CONCEPT: Helper function to execute a list as code
// This is reused from the main evaluator but specific to eval builtin
fn execute_list(list: &Value, interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Converting list to vector of values
    let mut current = list.clone();
    let mut items = Vec::new();

    // RUST CONCEPT: Traversing cons cell lists
    loop {
        match current {
            Value::Pair(car, cdr) => {
                items.push((*car).clone());
                current = (*cdr).clone();
            },
            Value::Nil => break,
            _ => {
                // RUST CONCEPT: Single values can be "executed" too
                // If it's not a proper list, just execute the single value
                use crate::evaluator::execute;
                return execute(&current, interp);
            }
        }
    }

    // RUST CONCEPT: Execute each element in sequence
    use crate::evaluator::execute;
    for item in items {
        execute(&item, interp)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_eval_builtin_simple() {
        let mut interp = setup_interpreter();

        // Create a simple arithmetic expression: [2 3 +]
        let plus_atom = interp.intern_atom("+");
        let expr = interp.make_list(vec![
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Atom(plus_atom),
        ]);

        interp.push(expr);
        eval_builtin(&mut interp).unwrap();

        // Should have evaluated to 5.0
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_eval_builtin_single_value() {
        let mut interp = setup_interpreter();

        // Test eval with a single number (should just push it)
        interp.push(Value::Number(42.0));
        eval_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));

        // Test eval with a single atom (should execute it)
        interp.push(Value::Number(10.0));
        interp.push(Value::Number(5.0));
        let plus_atom = interp.intern_atom("+");
        interp.push(Value::Atom(plus_atom));
        eval_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 15.0));
    }

    #[test]
    fn test_eval_builtin_empty_list() {
        let mut interp = setup_interpreter();

        // Test eval with empty list (should do nothing)
        interp.push(Value::Nil);
        eval_builtin(&mut interp).unwrap();

        // Stack should be empty
        let result = interp.pop();
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_eval_builtin_nested_operations() {
        let mut interp = setup_interpreter();

        // Create a more complex expression: [5 [2 3 +] eval *]
        // This should: push 5, evaluate [2 3 +] to get 5, then multiply: 5 * 5 = 25
        let plus_atom = interp.intern_atom("+");
        let inner_expr = interp.make_list(vec![
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Atom(plus_atom),
        ]);

        let eval_atom = interp.intern_atom("eval");
        let mul_atom = interp.intern_atom("*");
        let outer_expr = interp.make_list(vec![
            Value::Number(5.0),
            inner_expr,
            Value::Atom(eval_atom),
            Value::Atom(mul_atom),
        ]);

        interp.push(outer_expr);
        eval_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 25.0));
    }

    #[test]
    fn test_eval_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = eval_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_eval_builtin_execution_error() {
        let mut interp = setup_interpreter();

        // Create expression with undefined word: [nonexistent]
        let nonexistent_atom = interp.intern_atom("nonexistent");
        let expr = interp.make_list(vec![
            Value::Atom(nonexistent_atom),
        ]);

        interp.push(expr);
        let result = eval_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::UndefinedWord(_))));
    }
}