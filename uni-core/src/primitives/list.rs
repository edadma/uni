// List construction from multiple stack elements
// Creates a list from the top 'count' stack elements

use crate::compat::{ToString, Vec};
use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;

#[cfg(target_os = "none")]
use num_traits::Float;

// Stack-based list: ( element1 element2 ... elementN count -- list )
// Creates a list from the top 'count' stack elements in reverse order
pub fn list_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let count_value = interp.pop_number()?;

    // Input validation
    if count_value < 0.0 || count_value.fract() != 0.0 {
        return Err(RuntimeError::TypeError(
            "list count must be a non-negative integer".to_string(),
        ));
    }

    let count = count_value as usize;

    // Collect elements from stack
    let mut elements = Vec::with_capacity(count);
    for _ in 0..count {
        elements.push(interp.pop()?);
    }

    // Elements are in reverse order (stack is LIFO)
    // We need to reverse them to get the correct list order
    elements.reverse();

    // Use interpreter's list construction helper
    let list = interp.make_list(elements);
    interp.push(list);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_list_impl() {
        let mut interp = AsyncInterpreter::new();

        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(3.0)); // count
        list_impl(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        match result {
            Value::Pair(car, cdr) => {
                assert!(matches!(car.as_ref(), Value::Number(n) if *n == 1.0));
                match cdr.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Number(n) if *n == 2.0));
                        match cdr2.as_ref() {
                            Value::Pair(car3, cdr3) => {
                                assert!(matches!(car3.as_ref(), Value::Number(n) if *n == 3.0));
                                assert!(matches!(cdr3.as_ref(), Value::Nil));
                            }
                            _ => panic!("Expected pair for third element"),
                        }
                    }
                    _ => panic!("Expected pair for second element"),
                }
            }
            _ => panic!("Expected list (pair)"),
        }
    }
}
