// Stack manipulation primitives (dup, drop, swap, over, rot)

use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;

// Dup: ( a -- a a )
pub fn dup_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    interp.push(value.clone());
    interp.push(value);
    Ok(())
}

// Drop: ( a -- )
pub fn drop_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    interp.pop()?;
    Ok(())
}

// Swap: ( a b -- b a )
pub fn swap_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;
    interp.push(b);
    interp.push(a);
    Ok(())
}

// Over: ( a b -- a b a )
pub fn over_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;
    interp.push(a.clone());
    interp.push(b);
    interp.push(a);
    Ok(())
}

// Rot: ( a b c -- b c a )
pub fn rot_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let c = interp.pop()?;
    let b = interp.pop()?;
    let a = interp.pop()?;
    interp.push(b);
    interp.push(c);
    interp.push(a);
    Ok(())
}
