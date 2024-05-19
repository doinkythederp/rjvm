use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use core::{error::Error, fmt, fmt::Formatter};

use bytes::Bytes;

use crate::io::JvmIo;

/// Models an entry in the class path, i.e. a single Jar or directory
pub trait ClassPathEntry: fmt::Debug {
    // TODO: should `class_name` be a newtype?
    fn resolve(&self, fs: &dyn JvmIo, class_name: &str)
        -> Result<Option<Bytes>, ClassLoadingError>;
}

/// Error returned when loading a class does not work
#[derive(Debug)]
pub struct ClassLoadingError {
    message: String,
    source: Box<dyn Error>,
}

impl ClassLoadingError {
    pub fn new(error: impl Error + 'static) -> Self {
        Self {
            message: error.to_string(),
            source: Box::new(error),
        }
    }
}

impl fmt::Display for ClassLoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ClassLoadingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

// Test utilities used by multiple files
#[cfg(all(test, feature = "std"))]
pub mod tests {
    use crate::{class_path_entry::ClassPathEntry, io::JvmIo};

    pub fn assert_can_find_class(entry: &impl ClassPathEntry, fs: &impl JvmIo, class_name: &str) {
        let buf = entry
            .resolve(fs, class_name)
            .expect("should have been able to read file")
            .expect("should have been able to find file");
        let magic_number =
            u32::from_be_bytes(buf[0..4].try_into().expect("file should have 4 bytes"));
        assert_eq!(0xCAFEBABE, magic_number);
    }

    pub fn assert_cannot_find_class(
        entry: &impl ClassPathEntry,
        fs: &impl JvmIo,
        class_name: &str,
    ) {
        assert!(entry
            .resolve(fs, class_name)
            .expect("should not have had any errors")
            .is_none());
    }
}
