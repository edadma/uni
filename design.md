# Uni Language - Comprehensive Design & Implementation Guide

## 🎯 Project Overview

**Uni** is a homoiconic stack-based programming language that unifies code and data. It combines Forth's immediate execution model with Lisp's powerful data structures, implemented in Rust with automatic memory management via reference counting.

**Primary Goal**: Learn Rust deeply by building a real interpreter, with extensive comments explaining Rust concepts at the language level.

## 🏗️ Language Design

### Core Philosophy
- **Homoiconic**: Code and data are the same thing - no special syntax that can't be represented as data
- **Stack-based**: Operations work on a stack like Forth
- **Immediate execution**: Atoms execute by default unless quoted
- **Unified**: Only three fundamental data types, but they compose to create everything

### Data Types

1. **Numbers**: `42`, `3.14` (f64 internally)
2. **Atoms**: `hello`, `+`, `fibonacci` (interned Rc<str>)
3. **Pairs**: `[a . b]` (cons cells like Scheme)
4. **Nil**: `[]` or explicit nil (marks end of proper lists)

### Syntax & Semantics

#### Execution Model
```uni
42          # Numbers push themselves onto stack
hello       # Atoms look up definition and execute
'hello      # Quoted atoms push the atom without executing  
[1 2 +]     # Lists push themselves as data (quotation)
[1 2 +] eval # Execute the list: pushes 1, 2, then adds
```

#### List Structures (Cons Cells)
```uni
# Proper lists (end in nil):
[1 2 3]           # Sugar for [1 . [2 . [3 . nil]]]
[]                # Empty list (nil)

# Improper lists (explicit pairs):
[a . b]           # Just a pair, doesn't end in nil
[1 . [2 . 3]]     # Nested pairs, not a proper list
```

#### Word Definition
```uni
'square [dup *] def     # Define square as duplicate and multiply
5 square                # -> 25

'factorial [            # Multi-line definition
  dup 1 <= [drop 1] [
    dup 1 - factorial *
  ] if
] def
```

#### Comments
```uni
5 3 +    \ Comments use backslash (handled by tokenizer)
dup *    \ So they don't break homoiconicity
```

### Key Design Decisions

1. **Comments in tokenizer**: Backslash comments are stripped during tokenization, never become data structures
2. **Atoms execute by default**: Rarely need `eval`, just execute directly
3. **Quote to prevent execution**: `'atom` pushes atom without executing
4. **Pairs not vectors**: Lists are cons cells for structural sharing and Lisp-style operations
5. **Reference counting**: Rc<T> for automatic memory management without GC pauses

## 🦀 Rust Implementation

### Current Code Structure

The implementation is in `src/main.rs` with extensive learning-focused comments:

#### Value Enum
```rust
#[derive(Debug, Clone)]
enum Value {
    Number(f64),                    // Direct storage, no heap allocation
    Atom(Rc<str>),                  // Reference-counted interned strings
    Pair(Rc<Value>, Rc<Value>),     // Cons cells: (head, tail)
    Nil,                            // Empty list marker
    Builtin(fn(&mut Interpreter) -> Result<(), RuntimeError>), // Function pointers
}
```

#### Error Handling
```rust
#[derive(Debug)]
enum RuntimeError {
    StackUnderflow,        // Not enough items on stack for operation
    TypeError(String),     // Wrong type for operation (e.g., + needs numbers)
    UndefinedWord(String), // Atom not found in dictionary
}
```

#### Interpreter State
```rust
struct Interpreter {
    stack: Vec<Value>,                      // The main stack for computation
    dictionary: HashMap<Rc<str>, Value>,    // Word definitions (Forth dictionary)
    atoms: HashMap<String, Rc<str>>,        // Atom interning table
}
```

### Key Rust Concepts Demonstrated

1. **Reference Counting**: `Rc<T>` allows sharing data without copying
    - `Rc::clone()` is cheap (just increments counter)
    - Multiple lists can share the same tail
    - Automatic cleanup when last reference drops

2. **Error Handling**: `Result<T, E>` instead of exceptions
    - `?` operator for early returns on errors
    - Explicit handling of all error cases
    - No hidden control flow

3. **Pattern Matching**: Safe way to extract enum variants
    - `match` expressions handle all cases
    - Compiler ensures exhaustiveness
    - No invalid casts or null pointer dereferences

4. **Borrowing & Ownership**:
    - `&mut self` for methods that modify state
    - Borrow checker prevents data races at compile time
    - Split complex expressions to avoid double borrows

5. **Collections**:
    - `Vec<T>` for growable arrays (the stack)
    - `HashMap<K, V>` for fast lookups (dictionary)
    - Automatic memory management for collections

### Borrow Checker Learning Moment

We hit a classic borrow checker error:
```rust
// ERROR: Double mutable borrow
interp.push(Value::Atom(interp.intern_atom("text")));

// FIX: Split into separate statements
let atom = interp.intern_atom("text");  // First borrow ends here
interp.push(Value::Atom(atom));         // Second borrow starts here
```

This teaches how Rust prevents memory safety issues at compile time.

## 🔧 Current Implementation Status

### ✅ Completed
- [x] Basic `Value` enum with all variants
- [x] `RuntimeError` with proper error types
- [x] `Interpreter` struct with stack and dictionary
- [x] Atom interning system for memory efficiency
- [x] Stack operations: push, pop, pop_number
- [x] List construction helper (`make_list`)
- [x] Function pointers as `Builtin` values
- [x] Comprehensive tests
- [x] Example builtin function (`+` addition)
- [x] All dead code warnings eliminated

### 🚧 Next Steps (In Priority Order)

1. **Tokenizer**
    - Parse text into tokens: numbers, atoms, brackets, dots
    - Handle backslash comments
    - Unicode support for atom names
    - *Rust concepts: iterators, string slicing, char processing*

2. **Parser**
    - Convert tokens to `Value` structures
    - Handle both `[a b c]` and `[a . b]` syntax
    - Proper error reporting with line/column numbers
    - *Rust concepts: recursive parsing, error propagation*

3. **Core Builtins**
    - Stack operations: `dup`, `swap`, `drop`, `rot`
    - Arithmetic: `+`, `-`, `*`, `/`
    - List operations: `head`, `tail`, `cons`
    - Logic: `if`, `def`, `eval`
    - *Rust concepts: function traits, macros for builtin definitions*

4. **Evaluation Engine**
    - Execute atoms by looking up in dictionary
    - Handle quoted atoms
    - Evaluate lists when requested
    - *Rust concepts: recursive evaluation, trait objects*

5. **REPL & File Loading**
    - Read-eval-print loop
    - Load and execute Uni source files
    - *Rust concepts: I/O, error handling, iterators*

## 📚 Educational Approach

### Learning-First Comments
Every piece of code includes comments explaining:
- **What** the Rust syntax means
- **Why** we chose this approach
- **How** it relates to memory safety
- **When** you'd use similar patterns

Example:
```rust
// Vec::pop returns Option<T> - Some(value) if not empty, None if empty
// .ok_or() converts Option to Result for consistent error handling
self.stack.pop().ok_or(RuntimeError::StackUnderflow)
```

### Incremental Complexity
- Start with basic concepts (ownership, borrowing)
- Add complexity gradually (Rc, HashMap, function pointers)
- Real-world examples before abstract explanations
- Working code at every step

### Common Rust Patterns Covered
- Error handling with `Result` and `?`
- Smart pointers (`Rc<T>` for sharing)
- Pattern matching for type safety
- Iterator methods (`.fold()`, `.rev()`, `.into_iter()`)
- Trait derivation (`#[derive(Debug, Clone)]`)
- Module organization and testing

## 🎮 Example Uni Programs

### Basic Stack Operations
```uni
5 3 +           # -> 8
dup *           # -> 64 (8 squared)
```

### List Processing
```uni
[1 2 3] head    # -> 1
[1 2 3] tail    # -> [2 3]  
4 [1 2 3] cons  # -> [4 1 2 3]
```

### Word Definition
```uni
'double [2 *] def
'square [dup *] def
5 double square     # -> 100
```

### Conditional Logic
```uni
'abs [dup 0 < [neg] [] if] def
-5 abs              # -> 5
```

### List Manipulation
```uni
'sum [
  dup nil? [drop 0] [
    dup head swap tail sum +
  ] if
] def

[1 2 3 4] sum       # -> 10
```

## 🔍 Testing Strategy

### Current Tests
- Atom interning verification (same string = same Rc)
- Stack operations (push/pop/underflow)
- List construction (proper cons cell structure)
- Error handling (type errors, stack underflow)

### Planned Tests
- Tokenizer: various input formats, edge cases
- Parser: syntax validation, error recovery
- Evaluation: builtin functions, word definitions
- Integration: full programs, file loading

## 💭 Design Philosophy Notes

### Why These Choices?

1. **Stack-based**: Simple execution model, no complex AST walking
2. **Homoiconic**: Code is data, enables powerful metaprogramming
3. **Reference counting**: Predictable memory usage, no GC pauses
4. **Cons cells**: Efficient sharing, natural for functional programming
5. **Immediate execution**: Direct like Forth, powerful like Lisp

### Trade-offs Made

- **Performance vs Safety**: Chose safety (Rc overhead vs raw pointers)
- **Simplicity vs Power**: Limited syntax but full computational power
- **Memory vs Speed**: Some duplication for easier borrowing
- **Learning vs Production**: Extra comments and error checking

## 🚀 Getting Started for Next Developer

1. **Review the current `main.rs`** - understand the Value enum and basic operations
2. **Run the tests** - `cargo test` to see what's working
3. **Experiment** - modify the `main()` function to try different operations
4. **Next task**: Implement the tokenizer to parse strings like `"5 3 + [1 2] 'hello"`

The foundation is solid - all the core data structures and memory management are in place. The next major piece is parsing text into our `Value` structures, which will unlock the ability to actually write and run Uni programs!

---

*This document captures the complete context of our Uni language development. The code in `main.rs` implements everything described here, with extensive Rust learning comments throughout.*