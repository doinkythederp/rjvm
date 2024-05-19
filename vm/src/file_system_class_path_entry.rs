use alloc::string::{String, ToString};
use core::{fmt, fmt::Formatter};

use bytes::Bytes;
use snafu::Snafu;
use unix_path::{Path, PathBuf};

use crate::{
    class_path_entry::{ClassLoadingError, ClassPathEntry},
    io::JvmIo,
};

/// Implementation of [ClassPathEntry] that searches for `.class` files,
/// using the given directory as the root package
#[derive(Debug)]
pub struct FileSystemClassPathEntry {
    base_directory: PathBuf,
}

impl FileSystemClassPathEntry {
    pub fn new<P: AsRef<Path>>(fs: &dyn JvmIo, path: P) -> Result<Self, InvalidDirectoryError> {
        let mut base_directory = PathBuf::new();
        base_directory.push(path);

        if !fs.exists(&base_directory) || !fs.is_dir(&base_directory) {
            Err(InvalidDirectoryError {
                path: base_directory.to_string_lossy().to_string(),
            })
        } else {
            Ok(Self { base_directory })
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(display("{inner}"))]
pub struct IoError {
    pub inner: no_std_io::io::Error,
}

impl ClassPathEntry for FileSystemClassPathEntry {
    fn resolve(
        &self,
        fs: &dyn JvmIo,
        class_name: &str,
    ) -> Result<Option<Bytes>, ClassLoadingError> {
        let mut candidate = self.base_directory.clone();
        candidate.push(class_name);
        candidate.set_extension("class");
        if fs.exists(&candidate) {
            fs.read(&candidate)
                .map(Bytes::from)
                .map(Some)
                .map_err(|inner| IoSnafu { inner }.build())
                .map_err(ClassLoadingError::new)
        } else {
            Ok(None)
        }
    }
}

/// Error returned when a directory is not valid
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidDirectoryError {
    path: String,
}

impl fmt::Display for InvalidDirectoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid directory: {}", self.path)
    }
}

impl core::error::Error for InvalidDirectoryError {}

#[cfg(all(test, feature = "std"))]
mod tests {
    use unix_path::PathBuf;

    use crate::{
        class_path_entry::tests::{assert_can_find_class, assert_cannot_find_class},
        file_system_class_path_entry::{FileSystemClassPathEntry, InvalidDirectoryError},
        io::StdJvmIo,
    };

    #[test]
    fn directory_not_found() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("foobar");
        assert_eq!(
            InvalidDirectoryError {
                path: path.to_string_lossy().to_string()
            },
            FileSystemClassPathEntry::new(&StdJvmIo, path)
                .expect_err("should not have found directory")
        );
    }

    #[test]
    fn file_system_class_path_entry_works() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources");
        let entry = FileSystemClassPathEntry::new(&StdJvmIo, path).expect("should find directory");

        assert_can_find_class(&entry, &StdJvmIo, "rjvm/NumericTypes");
        assert_can_find_class(&entry, &StdJvmIo, "rjvm/ControlFlow");
        assert_cannot_find_class(&entry, &StdJvmIo, "rjvm/Foo");
    }
}
