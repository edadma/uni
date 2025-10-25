// RUST CONCEPT: Uni Prelude Module
// This module contains Uni's prelude definitions - the standard words loaded at startup
// Following the Forth tradition, we define higher-level operations in terms of primitives

use crate::evaluator::execute_string;
use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;

// ASYNC CONCEPT: Async prelude initialization
// This function loads all prelude definitions into the interpreter asynchronously
pub async fn load_prelude(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Define prelude words using actual Uni code
    // This is much more natural than building def commands from string pairs
    // Each line is real Uni code that defines a word
    // RUST CONCEPT: Multi-line string for multiple definitions
    // Each definition is actual Uni code that uses def naturally
    let prelude_code = r#"
        \\ Stack manipulation words
        'swap [1 roll] def
        "( a b -- b a ) Swap top two stack items" doc

        'dup [0 pick] def
        "( a -- a a ) Duplicate top stack item" doc

        'over [1 pick] def
        "( a b -- a b a ) Copy second stack item to top" doc

        'rot [2 roll] def
        "( a b c -- b c a ) Rotate third item to top" doc

        'nip [swap drop] def
        "( a b -- b ) Remove second stack item" doc

        'tuck [swap over] def
        "( a b -- b a b ) Copy top below second item" doc

        'nil? [[] =] def
        "( x -- bool ) Test if value is empty list" doc

        \\ I/O operations
        'cr [10 emit] def
        "( -- ) Print a newline character" doc

        \\ Logical operations
        'not [[false] [true] if] def
        "( x -- bool ) Logical negation of truthiness" doc

        \\ Arithmetic operations
        'negate [-1 *] def
        "( n -- -n ) Negate a number" doc

        \\ List processing primitives
        'length [
            dup nil?
            [drop 0]
            [cdr length 1 +]
            if
        ] def
        "( list -- n ) Calculate list length recursively" doc

        'list-ref [
            dup 0 =
            [drop car]
            [1 - swap cdr swap list-ref]
            if
        ] def
        "( list index -- element ) Get nth element (0-indexed)" doc

        'append [
            swap dup nil?
            [drop]
            [
                dup car
                swap cdr
                rot
                append
                cons
            ]
            if
        ] def
        "( list1 list2 -- list3 ) Concatenate two lists" doc

        'null? [null =] def
        "( x -- bool ) Test if value is null" doc

        'record? [type "record" =] def
        "( x -- bool ) Test if value is any record type" doc

        \\ Conditional duplication from Forth
        '?dup [
            dup truthy? [dup] [] if
        ] def
        "( x -- x x | x ) Duplicate if truthy, otherwise leave unchanged" doc

        \\ Variable operations (Forth-style)
        '1+ [1 +] def
        "( n -- n+1 ) Increment by 1" doc

        '1- [1 -] def
        "( n -- n-1 ) Decrement by 1" doc

        '+! [dup @ rot + swap !] def
        "( n var -- ) Add n to variable" doc

        'on [true swap !] def
        "( var -- ) Store true to variable" doc

        'off [false swap !] def
        "( var -- ) Store false to variable" doc

        \\ List iteration
        'each [
            >r                      \\ Move fn to return stack: list | fn
            dup nil?                \\ Check if list is empty: list bool | fn
            [
                drop r> drop        \\ Empty list: clean up list and fn
            ]
            [
                dup car            \\ list -> list head | fn
                r@                  \\ Get fn: list head fn | fn
                exec                \\ Execute fn: list ... | fn (fn consumes head, may leave results)
                cdr                \\ Get tail: ... tail | fn
                r> each             \\ Recurse: ... tail fn
            ]
            if
        ] def
        "( list [fn] -- ) Execute fn on each element of list (fn consumes argument, may leave results)" doc

        \\ Short-circuiting logical operations
        'and [
            swap                          \\ Move first quotation to top
            exec                          \\ Execute first quotation
            dup                           \\ Always duplicate the result
            [
                drop                      \\ Drop the duplicate, keep original
                exec                      \\ Execute second quotation
            ]
            [
                swap drop                 \\ If falsy, drop second quotation, keep falsy result
            ]
            if
        ] def
        "( [cond1] [cond2] -- result ) Short-circuit AND: executes cond2 only if cond1 is truthy" doc

        'or [
            swap                          \\ Move first quotation to top
            exec                          \\ Execute first quotation
            dup                           \\ Always duplicate the result
            [
                swap drop                 \\ If truthy, drop second quotation, keep result
            ]
            [
                drop                      \\ Drop the duplicate
                exec                      \\ If falsy, execute second quotation
            ]
            if
        ] def
        "( [cond1] [cond2] -- result ) Short-circuit OR: executes cond2 only if cond1 is falsy" doc

        \\ Control flow primitives
        'while [
            >r >r                         \\ move body and condition to return stack
            r@ exec                       \\ execute condition (copy from R-stack)
            [
                r> r> dup rot swap >r >r  \\ get body and move body and condition back to return stack
                exec                      \\ execute body
                r> r> while               \\ recursive call
            ]
            [ r> r> drop drop ]
            if
        ] def
        "( [condition] [body] -- ) Loop: executes body while condition returns truthy" doc

        \\ Date/time operations
        \\ Date record type with calendar components
        \\ The 'now' primitive (written in Rust) returns instances of this record type
        ["year" "month" "day" "hour" "minute" "second" "offset"] "date" make-record-type drop
    "#;

    // ASYNC CONCEPT: Execute the prelude code directly using async execute_string
    // This uses the normal execution path - no special handling needed
    execute_string(prelude_code, interp).await?;

    // RUST CONCEPT: Conditional compilation for feature-specific prelude
    // Complex number constants (only when complex_numbers feature is enabled)
    #[cfg(feature = "complex_numbers")]
    {
        let complex_prelude = r#"
            \\ Mathematical constants (complex numbers)
            'i 0+1i def
            "Imaginary unit constant (0+1i)" doc
        "#;
        execute_string(complex_prelude, interp).await?;
    }

    // RUST CONCEPT: Conditional compilation for platform-specific prelude
    // Hardware convenience wrappers for embedded targets
    #[cfg(target_os = "none")]
    {
        let hardware_prelude = r#"
            \\ Hardware convenience wrappers (embedded targets only)
            'button-a? [0 button-read] def
            "( -- bool ) Read button A state (true = pressed)" doc

            'button-b? [1 button-read] def
            "( -- bool ) Read button B state (true = pressed)" doc
        "#;
        execute_string(hardware_prelude, interp).await?;
    }

    Ok(())
}
