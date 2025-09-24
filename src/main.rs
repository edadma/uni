mod value;
mod interpreter;
mod builtins;
mod tokenizer;

use interpreter::Interpreter;
use value::{Value, RuntimeError};
use builtins::register_builtins;

fn main() {
    println!("Uni interpreter starting...");

    let mut interp = Interpreter::new();
    register_builtins(&mut interp);

    interp.push(Value::Number(42.0));
    println!("Pushed number: 42.0");

    let hello_atom = interp.intern_atom("hello");
    interp.push(Value::Atom(hello_atom));
    println!("Pushed atom: hello");

    let list = interp.make_list(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0)
    ]);
    interp.push(list);
    println!("Pushed list: [1 2 3] (as cons cells)");

    println!("Defined builtin: +");

    interp.push(Value::Number(5.0));
    interp.push(Value::Number(3.0));

    let plus_atom = interp.intern_atom("+");
    if let Some(Value::Builtin(func)) = interp.dictionary.get(&plus_atom) {
        match func(&mut interp) {
            Ok(()) => println!("Successfully called + builtin"),
            Err(e) => println!("Error calling +: {:?}", e),
        }
    }

    match interp.pop() {
        Ok(Value::Number(n)) => println!("Result: {}", n),
        Ok(other) => println!("Got non-number: {:?}", other),
        Err(e) => println!("Error: {:?}", e),
    }

    match interp.pop() {
        Ok(val) => println!("Unexpected value: {:?}", val),
        Err(RuntimeError::StackUnderflow) => println!("Caught stack underflow as expected"),
        Err(e) => println!("Unexpected error: {:?}", e),
    }

    interp.push(Value::Nil);
    println!("Pushed empty list (Nil)");

    let not_number_atom = interp.intern_atom("not-a-number");
    interp.push(Value::Atom(not_number_atom));
    match interp.pop_number() {
        Ok(n) => println!("Unexpected number: {}", n),
        Err(RuntimeError::TypeError(msg)) => println!("Caught type error: {}", msg),
        Err(e) => println!("Unexpected error: {:?}", e),
    }

    // Test string handling
    use std::rc::Rc;
    let string_val: Rc<str> = "Hello, Uni!".into();
    interp.push(Value::String(string_val));
    println!("Pushed string: \"Hello, Uni!\"");

    match interp.pop_string() {
        Ok(s) => println!("Retrieved string: \"{}\"", s),
        Err(e) => println!("Error retrieving string: {:?}", e),
    }
}