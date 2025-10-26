//! Linux/desktop hardware primitives
//!
//! Provides date/time access using chrono

use crate::interpreter::{AsyncInterpreter, DictEntry};
use crate::value::{RuntimeError, Value};
use crate::compat::{Rc, format};
use chrono::{Local, Datelike, Timelike, NaiveDateTime, NaiveDate, NaiveTime};

/// Register Linux-specific primitives with the interpreter
#[cfg(feature = "std")]
pub fn register_linux_primitives(interp: &mut AsyncInterpreter) {
    // Register now primitive
    let now_atom = interp.intern_atom("now");
    interp.dict_insert(now_atom.clone(), DictEntry {
        value: Value::AsyncBuiltin(now_impl),
        is_executable: true,
        doc: Some(Rc::<str>::from("( -- record ) Get current date/time as a record with fields: year month day hour minute second offset-minutes")),
    });

    // Register set-time primitive
    let set_time_atom = interp.intern_atom("set-time");
    interp.dict_insert(set_time_atom.clone(), DictEntry {
        value: Value::AsyncBuiltin(set_time_impl),
        is_executable: true,
        doc: Some(Rc::<str>::from("( year month day hour minute second -- ) Set system time (requires elevated privileges)")),
    });
}

/// Get current date/time
/// Stack: ( -- record )
/// Returns a record with fields: year, month, day, hour, minute, second, offset-minutes
#[cfg(feature = "std")]
pub fn now_impl(interp: &mut AsyncInterpreter) -> core::pin::Pin<crate::compat::Box<dyn core::future::Future<Output = Result<(), RuntimeError>> + '_>> {
    use crate::compat::Box;
    Box::pin(async move {
        use crate::evaluator::execute_string;

        // Get current local time using chrono
        let now = Local::now();

        // Get timezone offset in seconds, convert to minutes
        let offset_seconds = now.offset().local_minus_utc();
        let offset_minutes = offset_seconds / 60;

        // Ensure datetime record type exists
        crate::builtins::ensure_datetime_record_type(interp).await?;

        // Push field values in order: year month day hour minute second offset-minutes
        interp.push(Value::Int32(now.year()));
        interp.push(Value::Int32(now.month() as i32));
        interp.push(Value::Int32(now.day() as i32));
        interp.push(Value::Int32(now.hour() as i32));
        interp.push(Value::Int32(now.minute() as i32));
        interp.push(Value::Int32(now.second() as i32));
        interp.push(Value::Int32(offset_minutes));

        // Call make-datetime to construct the record
        execute_string("make-datetime", interp).await?;

        Ok(())
    })
}

/// Set system time
/// Stack: ( year month day hour minute second -- )
/// Note: On Linux, this requires root/sudo privileges
#[cfg(feature = "std")]
pub fn set_time_impl(interp: &mut AsyncInterpreter) -> core::pin::Pin<crate::compat::Box<dyn core::future::Future<Output = Result<(), RuntimeError>> + '_>> {
    use crate::compat::Box;
    Box::pin(async move {
    // Pop time components from stack
    let second = interp.pop_integer()? as u32;
    let minute = interp.pop_integer()? as u32;
    let hour = interp.pop_integer()? as u32;
    let day = interp.pop_integer()? as u32;
    let month = interp.pop_integer()? as u32;
    let year = interp.pop_integer()? as i32;

    // Validate ranges
    if month < 1 || month > 12 {
        return Err(RuntimeError::DomainError(format!("Invalid month: {}", month)));
    }
    if day < 1 || day > 31 {
        return Err(RuntimeError::DomainError(format!("Invalid day: {}", day)));
    }
    if hour > 23 {
        return Err(RuntimeError::DomainError(format!("Invalid hour: {}", hour)));
    }
    if minute > 59 {
        return Err(RuntimeError::DomainError(format!("Invalid minute: {}", minute)));
    }
    if second > 59 {
        return Err(RuntimeError::DomainError(format!("Invalid second: {}", second)));
    }

    // Create NaiveDateTime
    let date = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| RuntimeError::DomainError(format!("Invalid date: {}-{}-{}", year, month, day)))?;
    let time = NaiveTime::from_hms_opt(hour, minute, second)
        .ok_or_else(|| RuntimeError::DomainError(format!("Invalid time: {}:{}:{}", hour, minute, second)))?;
    let datetime = NaiveDateTime::new(date, time);

    // Convert to timestamp
    let timestamp = datetime.and_utc().timestamp();

    // Try to set system time using libc (requires root)
    #[cfg(target_os = "linux")]
    {
        unsafe {
            let tv = libc::timeval {
                tv_sec: timestamp,
                tv_usec: 0,
            };
            let result = libc::settimeofday(&tv as *const libc::timeval, std::ptr::null());
            if result != 0 {
                let errno = *libc::__errno_location();
                if errno == libc::EPERM {
                    return Err(RuntimeError::DomainError(
                        "Permission denied: setting system time requires root/sudo privileges".into()
                    ));
                } else {
                    return Err(RuntimeError::DomainError(
                        format!("Failed to set system time: errno {}", errno)
                    ));
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        return Err(RuntimeError::DomainError(
            "set-time is only supported on Linux".into()
        ));
    }

    Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_now() {
        let mut interp = AsyncInterpreter::new();
        register_linux_primitives(&mut interp);

        // Call now
        now_impl(&mut interp).await.unwrap();

        // Should have a datetime record on stack
        assert_eq!(interp.stack.len(), 1);

        let record = interp.stack.last().unwrap();
        match record {
            Value::Record { type_name, .. } => {
                assert_eq!(type_name.as_ref(), "datetime");
            }
            _ => panic!("Expected datetime record, got: {:?}", record),
        }
    }
}
