pub mod time;

#[derive(Debug, Ord, Eq, PartialOrd, PartialEq)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
}

pub const DEBUG_LEVEL: Level = Level::Debug;

macro_rules! _log_debug {
    ($($arg:tt)*) => {
        if $crate::log::DEBUG_LEVEL >= $crate::log::Level::Debug {
            $crate::log::print_log_message($crate::log::Level::Debug, format!($($arg)*))
        }
    };
}

macro_rules! _log_info {
    ($($arg:tt)*) => {
        if $crate::log::DEBUG_LEVEL >= $crate::log::Level::Info {
            $crate::log::print_log_message($crate::log::Level::Info, format!($($arg)*))
        }
    };
}

macro_rules! _log_warn {
    ($($arg:tt)*) => {
        if $crate::log::DEBUG_LEVEL >= $crate::log::Level::Warn {
            $crate::log::print_log_message($crate::log::Level::Warn, format!($($arg)*))
        }
     }
}

macro_rules! _log_error {
    ($($arg:tt)*) => {
        if $crate::log::DEBUG_LEVEL >= $crate::log::Level::Error {
            $crate::log::print_log_message($crate::log::Level::Error, format!($($arg)*))
        }
    }
}

pub(crate) use _log_debug as debug;
pub(crate) use _log_error as error;
pub(crate) use _log_info as info;

fn get_formatted_current_time() -> String {
    let time = time::get_current_time();

    format!(
        "{:02}/{:02}/{:04} {:02}:{:02}:{:02}",
        time.day, time.month, time.year, time.hour, time.minute, time.second
    )
}

pub fn print_log_message(level: Level, message: String) {
    let time = get_formatted_current_time();

    match level {
        Level::Debug => println!("[{}] [\x1B[94mDEBUG\x1B[0m] {}\x1B[0m", time, message),
        Level::Info => println!("[{}] [\x1B[92mINFO\x1B[0m] {}\x1B[0m", time, message),

        Level::Warn => println!("[{}] [\x1B[33mWARN\x1B[0m] {}\x1B[0m", time, message),
        Level::Error => println!("[{}] [\x1B[91mERROR\x1B[0m] {}\x1B[0m", time, message),
    }
}
