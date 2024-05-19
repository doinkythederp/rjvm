use crate::io::JvmIo;

/// Returns the current epoch as nano seconds
pub(crate) fn get_nano_time(io: &dyn JvmIo) -> i64 {
    io.duration_since_epoch().as_nanos() as i64
}

/// Returns the current epoch as milliseconds
pub(crate) fn get_current_time_millis(io: &dyn JvmIo) -> i64 {
    io.duration_since_epoch().as_millis() as i64
}
