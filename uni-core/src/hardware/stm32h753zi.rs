//! STM32H753ZI hardware primitives
//!
//! Provides RTC access using embassy-stm32

use crate::interpreter::{AsyncInterpreter, DictEntry};
use crate::value::{RuntimeError, Value};
use crate::compat::Rc;

/// Register STM32-specific primitives with the interpreter
#[cfg(feature = "target-stm32h753zi")]
pub fn register_stm32_primitives(interp: &mut AsyncInterpreter) {
    // Register now primitive
    let now_atom = interp.intern_atom("now");
    interp.dict_insert(now_atom.clone(), DictEntry {
        value: Value::AsyncBuiltin(now_impl),
        is_executable: true,
        doc: Some(Rc::<str>::from("( -- record ) Get current date/time from RTC as a record with fields: year month day hour minute second offset-minutes")),
    });

    // Register set-time primitive
    let set_time_atom = interp.intern_atom("set-time");
    interp.dict_insert(set_time_atom.clone(), DictEntry {
        value: Value::AsyncBuiltin(set_time_impl),
        is_executable: true,
        doc: Some(Rc::<str>::from("( year month day hour minute second -- ) Set RTC time")),
    });
}

/// Get current date/time from RTC
/// Stack: ( -- record )
/// Returns a record with fields: year, month, day, hour, minute, second, offset-minutes
#[cfg(feature = "target-stm32h753zi")]
pub fn now_impl(interp: &mut AsyncInterpreter) -> core::pin::Pin<crate::compat::Box<dyn core::future::Future<Output = Result<(), RuntimeError>> + '_>> {
    use crate::compat::Box;
    Box::pin(async move {
        use crate::evaluator::execute_string;
        use crate::platform::Platform;

        // Get RTC from platform
        let rtc = match &interp.platform {
            Platform::Stm32(platform) => {
                platform.rtc.as_ref().ok_or_else(|| {
                    RuntimeError::DomainError("RTC not available".into())
                })?
            }
            _ => return Err(RuntimeError::DomainError("now is only available on STM32 platform".into())),
        };

        // Read current time from RTC
        let datetime = {
            #[cfg(target_os = "none")]
            {
                rtc.borrow().now().map_err(|_| {
                    RuntimeError::DomainError("Failed to read RTC".into())
                })?
            }
            #[cfg(not(target_os = "none"))]
            {
                rtc.lock().unwrap().now().map_err(|_| {
                    RuntimeError::DomainError("Failed to read RTC".into())
                })?
            }
        };

        // Ensure datetime record type exists
        crate::builtins::ensure_datetime_record_type(interp).await?;

        // Push field values in order: year month day hour minute second offset-minutes
        // Note: embassy-stm32 RTC doesn't track timezone, so we use 0 for offset
        interp.push(Value::Int32(datetime.year() as i32));
        interp.push(Value::Int32(datetime.month() as i32));
        interp.push(Value::Int32(datetime.day() as i32));
        interp.push(Value::Int32(datetime.hour() as i32));
        interp.push(Value::Int32(datetime.minute() as i32));
        interp.push(Value::Int32(datetime.second() as i32));
        interp.push(Value::Int32(0)); // offset-minutes (RTC doesn't track timezone)

        // Call make-datetime to construct the record
        execute_string("make-datetime", interp).await?;

        Ok(())
    })
}

/// Set RTC time
/// Stack: ( year month day hour minute second -- )
#[cfg(feature = "target-stm32h753zi")]
pub fn set_time_impl(interp: &mut AsyncInterpreter) -> core::pin::Pin<crate::compat::Box<dyn core::future::Future<Output = Result<(), RuntimeError>> + '_>> {
    use crate::compat::Box;
    Box::pin(async move {
        use crate::platform::Platform;

        // Pop time components from stack
        let second = interp.pop_integer()? as u8;
        let minute = interp.pop_integer()? as u8;
        let hour = interp.pop_integer()? as u8;
        let day = interp.pop_integer()? as u8;
        let month = interp.pop_integer()? as u8;
        let year = interp.pop_integer()? as u16;

        // Validate ranges
        if month < 1 || month > 12 {
            return Err(RuntimeError::DomainError(crate::compat::format!("Invalid month: {}", month)));
        }
        if day < 1 || day > 31 {
            return Err(RuntimeError::DomainError(crate::compat::format!("Invalid day: {}", day)));
        }
        if hour > 23 {
            return Err(RuntimeError::DomainError(crate::compat::format!("Invalid hour: {}", hour)));
        }
        if minute > 59 {
            return Err(RuntimeError::DomainError(crate::compat::format!("Invalid minute: {}", minute)));
        }
        if second > 59 {
            return Err(RuntimeError::DomainError(crate::compat::format!("Invalid second: {}", second)));
        }

        // Get RTC from platform
        let rtc = match &interp.platform {
            Platform::Stm32(platform) => {
                platform.rtc.as_ref().ok_or_else(|| {
                    RuntimeError::DomainError("RTC not available".into())
                })?
            }
            _ => return Err(RuntimeError::DomainError("set-time is only available on STM32 platform".into())),
        };

        // Create DateTime and set RTC
        {
            use embassy_stm32::rtc::{DateTime, DayOfWeek};

            // Calculate day of week (simplified - just use Monday for now)
            // TODO: Properly calculate day of week from date
            let day_of_week = DayOfWeek::Monday;

            let datetime = DateTime::from(year, month, day, day_of_week, hour, minute, second, 0)
                .map_err(|_| RuntimeError::DomainError("Invalid date/time values".into()))?;

            #[cfg(target_os = "none")]
            { rtc.borrow_mut().set_datetime(datetime).map_err(|_| {
                RuntimeError::DomainError("Failed to set RTC time".into())
            })?; }
            #[cfg(not(target_os = "none"))]
            { rtc.lock().unwrap().set_datetime(datetime).map_err(|_| {
                RuntimeError::DomainError("Failed to set RTC time".into())
            })?; }
        }

        Ok(())
    })
}
