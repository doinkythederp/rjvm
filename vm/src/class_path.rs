use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

use bytes::Bytes;
use log::debug;
use snafu::Snafu;

use crate::{
    class_path_entry::{ClassLoadingError, ClassPathEntry},
    file_system_class_path_entry::FileSystemClassPathEntry,
    io::JvmIo,
    jar_file_class_path_entry::JarFileClassPathEntry,
};

/// Models a class path, i.e. a list of [ClassPathEntry]
#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct ClassPath {
    entries: Vec<Box<dyn ClassPathEntry>>,
}

/// Error that models the fact that a class path entry was not valid
#[derive(Snafu, Debug, PartialEq)]
pub enum ClassPathParseError {
    #[snafu(display("invalid classpath entry: {entry}"))]
    InvalidEntry { entry: String },
}

impl ClassPath {
    /// Parses and adds class path entries.
    /// These should be separated by a colon (:), just like in a real JVM.
    pub fn push(&mut self, fs: &dyn JvmIo, string: &str) -> Result<(), ClassPathParseError> {
        let mut entries_to_add: Vec<Box<dyn ClassPathEntry>> = Vec::new();
        for entry in string.split(':') {
            debug!("trying to parse class path entry {}", entry);
            let parsed_entry = Self::try_parse_entry(fs, entry)?;
            entries_to_add.push(parsed_entry);
        }
        self.entries.append(&mut entries_to_add);
        Ok(())
    }

    fn try_parse_entry(
        fs: &dyn JvmIo,
        path: &str,
    ) -> Result<Box<dyn ClassPathEntry>, ClassPathParseError> {
        Self::try_parse_entry_as_jar(fs, path)
            .or_else(|_| Self::try_parse_entry_as_directory(fs, path))
    }

    fn try_parse_entry_as_jar(
        fs: &dyn JvmIo,
        path: &str,
    ) -> Result<Box<dyn ClassPathEntry>, ClassPathParseError> {
        let entry = JarFileClassPathEntry::new(fs, path).map_err(|_| {
            ClassPathParseError::InvalidEntry {
                entry: path.to_string(),
            }
        })?;
        Ok(Box::new(entry))
    }

    fn try_parse_entry_as_directory(
        fs: &dyn JvmIo,
        path: &str,
    ) -> Result<Box<dyn ClassPathEntry>, ClassPathParseError> {
        let entry = FileSystemClassPathEntry::new(fs, path).map_err(|_| {
            ClassPathParseError::InvalidEntry {
                entry: path.to_string(),
            }
        })?;
        Ok(Box::new(entry))
    }

    /// Attempts to resolve a class from the various entries.
    /// Stops at the first entry that has a match or an error.
    pub fn resolve(
        &self,
        fs: &dyn JvmIo,
        class_name: &str,
    ) -> Result<Option<Bytes>, ClassLoadingError> {
        for entry in self.entries.iter() {
            debug!("looking up class {} in {:?}", class_name, entry);
            let entry_result = entry.resolve(fs, class_name)?;
            if let Some(class_bytes) = entry_result {
                return Ok(Some(class_bytes));
            }
        }
        Ok(None)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloc::format;

    use super::ClassPath;
    use crate::io::{JvmIo, StdJvmIo};

    #[test]
    fn can_parse_valid_classpath_entries() {
        let dir = env!("CARGO_MANIFEST_DIR");
        let mut class_path: ClassPath = Default::default();
        class_path
            .push(
                &StdJvmIo,
                &format!("{dir}/tests/resources/sample.jar:{dir}/tests/resources",),
            )
            .expect("should be able to parse classpath");
        assert_can_find_class(&class_path, &StdJvmIo, "rjvm/NumericTypes"); // From jar
        assert_can_find_class(&class_path, &StdJvmIo, "rjvm/SimpleMain"); // From directory
        assert_cannot_find_class(&class_path, &StdJvmIo, "foo");
    }

    fn assert_can_find_class(class_path: &ClassPath, fs: &dyn JvmIo, class_name: &str) {
        let buf = class_path
            .resolve(fs, class_name)
            .expect("should not have had any errors")
            .expect("should have been able to find file");
        let magic_number =
            u32::from_be_bytes(buf[0..4].try_into().expect("file should have 4 bytes"));
        assert_eq!(0xCAFEBABE, magic_number);
    }

    fn assert_cannot_find_class(class_path: &ClassPath, fs: &dyn JvmIo, class_name: &str) {
        assert!(class_path
            .resolve(fs, class_name)
            .expect("should not have had any errors")
            .is_none());
    }
}
