// Vector (array) primitives inspired by Scheme vectors
// Provides dense, indexed collections alongside existing list structures

use crate::compat::{format, Rc, ToString, Vec};
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

#[cfg(target_os = "none")]
use num_traits::Float;

fn expect_array(value: Value, op_name: &str) -> Result<Rc<RefCell<Vec<Value>>>, RuntimeError> {
    match value {
        Value::Array(array) => Ok(array),
        _ => Err(RuntimeError::TypeError(format!(
            "{} expects an array",
            op_name
        ))),
    }
}

fn expect_index(index: f64, op_name: &str) -> Result<usize, RuntimeError> {
    if index < 0.0 || index.fract() != 0.0 {
        return Err(RuntimeError::TypeError(format!(
            "{} index must be a non-negative integer",
            op_name
        )));
    }
    Ok(index as usize)
}

fn collect_list_elements(list: Value) -> Result<Vec<Value>, RuntimeError> {
    let mut elements = Vec::new();
    let mut current = list;

    loop {
        match current {
            Value::Pair(car, cdr) => {
                elements.push((*car).clone());
                current = (*cdr).clone();
            }
            Value::Nil => break,
            _ => {
                return Err(RuntimeError::TypeError(
                    "list->vector expects a proper list".to_string(),
                ));
            }
        }
    }

    Ok(elements)
}

// Stack effect: ( elementN ... element1 count -- vector )
// Collects top count items (in insertion order) into a new vector
pub fn vector_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let count_value = interp.pop_number()?;
    if count_value < 0.0 || count_value.fract() != 0.0 {
        return Err(RuntimeError::TypeError(
            "vector count must be a non-negative integer".to_string(),
        ));
    }

    let count = count_value as usize;
    let mut elements = Vec::with_capacity(count);
    for _ in 0..count {
        elements.push(interp.pop()?);
    }
    elements.reverse();

    interp.push(interp.make_array(elements));
    Ok(())
}

// Stack effect: ( count fill -- vector )
// Creates vector with count copies of fill value
pub fn make_vector_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let fill_value = interp.pop()?;
    let count_value = interp.pop_number()?;
    if count_value < 0.0 || count_value.fract() != 0.0 {
        return Err(RuntimeError::TypeError(
            "make-vector count must be a non-negative integer".to_string(),
        ));
    }
    let count = count_value as usize;

    let mut elements = Vec::with_capacity(count);
    for _ in 0..count {
        elements.push(fill_value.clone());
    }

    interp.push(interp.make_array(elements));
    Ok(())
}

// Stack effect: ( vector -- length )
pub fn vector_length_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let vector_value = interp.pop()?;
    let array = expect_array(vector_value, "vector-length")?;
    let len = array.borrow().len();
    interp.push(Value::Number(len as f64));
    Ok(())
}

// Stack effect: ( vector index -- element )
pub fn vector_ref_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let index_value = interp.pop_number()?;
    let index = expect_index(index_value, "vector-ref")?;
    let vector_value = interp.pop()?;
    let array = expect_array(vector_value, "vector-ref")?;

    let elements = array.borrow();
    let element = elements.get(index).cloned().ok_or_else(|| {
        RuntimeError::TypeError(format!(
            "vector-ref index {} out of bounds for length {}",
            index,
            elements.len()
        ))
    })?;

    interp.push(element);
    Ok(())
}

// Stack effect: ( value vector index -- )
// Following Forth convention where value comes first
pub fn vector_set_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let index_value = interp.pop_number()?;
    let index = expect_index(index_value, "vector-set!")?;
    let vector_value = interp.pop()?;
    let array = expect_array(vector_value, "vector-set!")?;
    let new_value = interp.pop()?;

    let mut elements = array.borrow_mut();
    if index >= elements.len() {
        return Err(RuntimeError::TypeError(format!(
            "vector-set! index {} out of bounds for length {}",
            index,
            elements.len()
        )));
    }
    elements[index] = new_value;
    Ok(())
}

// Stack effect: ( vector -- list )
pub fn vector_to_list_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let vector_value = interp.pop()?;
    let array = expect_array(vector_value, "vector->list")?;
    let elements = array.borrow();
    let list_elements: Vec<Value> = elements.iter().cloned().collect();
    interp.push(interp.make_list(list_elements));
    Ok(())
}

// Stack effect: ( list -- vector )
pub fn list_to_vector_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let list_value = interp.pop()?;
    let elements = collect_list_elements(list_value)?;
    interp.push(interp.make_array(elements));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::AsyncInterpreter;

    fn setup_interpreter() -> AsyncInterpreter {
        AsyncInterpreter::new()
    }

    fn unwrap_array(value: Value) -> Rc<RefCell<Vec<Value>>> {
        match value {
            Value::Array(array) => array,
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_vector_basic() {
        let mut interp = setup_interpreter();

        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(3.0));
        vector_impl(&mut interp).unwrap();

        let array_rc = unwrap_array(interp.pop().unwrap());
        let array = array_rc.borrow();
        assert_eq!(array.len(), 3);
        assert!(matches!(array[0], Value::Number(n) if n == 1.0));
        assert!(matches!(array[1], Value::Number(n) if n == 2.0));
        assert!(matches!(array[2], Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_make_vector() {
        let mut interp = setup_interpreter();

        interp.push(Value::Number(4.0));
        interp.push(Value::String("fill".into()));
        make_vector_impl(&mut interp).unwrap();

        let array_rc = unwrap_array(interp.pop().unwrap());
        let array = array_rc.borrow();
        assert_eq!(array.len(), 4);
        for element in array.iter() {
            assert!(matches!(element, Value::String(s) if s.as_ref() == "fill"));
        }
    }

    #[test]
    fn test_vector_length() {
        let mut interp = setup_interpreter();

        interp.push(Value::Number(5.0));
        interp.push(Value::Number(6.0));
        interp.push(Value::Number(2.0));
        vector_impl(&mut interp).unwrap();
        vector_length_impl(&mut interp).unwrap();

        let len_value = interp.pop().unwrap();
        assert!(matches!(len_value, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_vector_ref() {
        let mut interp = setup_interpreter();

        interp.push(Value::Number(7.0));
        interp.push(Value::Number(8.0));
        interp.push(Value::Number(2.0));
        vector_impl(&mut interp).unwrap();

        let vector_value = interp.pop().unwrap();
        interp.push(vector_value.clone());
        interp.push(Value::Number(1.0));
        vector_ref_impl(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0));
    }

    #[test]
    fn test_vector_set() {
        let mut interp = setup_interpreter();

        interp.push(Value::Number(1.0));
        interp.push(Value::Number(1.0));
        vector_impl(&mut interp).unwrap();

        let vector_value = interp.pop().unwrap();
        interp.push(Value::Number(42.0));
        interp.push(vector_value.clone());
        interp.push(Value::Number(0.0));
        vector_set_impl(&mut interp).unwrap();

        let array_rc = unwrap_array(vector_value);
        let array = array_rc.borrow();
        assert!(matches!(array[0], Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_vector_to_list() {
        let mut interp = setup_interpreter();

        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(2.0));
        vector_impl(&mut interp).unwrap();

        let vector_value = interp.pop().unwrap();
        interp.push(vector_value);
        vector_to_list_impl(&mut interp).unwrap();

        let list_value = interp.pop().unwrap();
        match list_value {
            Value::Pair(car, cdr) => {
                assert!(matches!(car.as_ref(), Value::Number(n) if *n == 1.0));
                match cdr.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Number(n) if *n == 2.0));
                        assert!(matches!(cdr2.as_ref(), Value::Nil));
                    }
                    _ => panic!("Expected second element in list"),
                }
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_list_to_vector() {
        let mut interp = setup_interpreter();

        let list = interp.make_list(vec![Value::Number(5.0), Value::Boolean(true)]);
        interp.push(list);
        list_to_vector_impl(&mut interp).unwrap();

        let array_rc = unwrap_array(interp.pop().unwrap());
        let array = array_rc.borrow();
        assert_eq!(array.len(), 2);
        assert!(matches!(array[0], Value::Number(n) if n == 5.0));
        assert!(matches!(array[1], Value::Boolean(true)));
    }
}
