use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::{
    cell::RefCell,
    cmp,
    fmt::{Debug, Formatter},
    str::Utf8Error,
};

use bytes::{Buf, Bytes};
use hashbrown::HashMap;
use miniz_oxide::inflate::decompress_to_vec;
use snafu::{ResultExt, Snafu};
use unix_path::{Path, PathBuf};
use zip::{CompressMethod, LocalFileOps, ParsingError, SequentialParser as ZipArchive};

use crate::{
    class_path_entry::{ClassLoadingError, ClassPathEntry},
    io::JvmIo,
};

struct ZipData {
    pub buf: Bytes,
}

impl zip::Read for ZipData {
    fn read(&mut self, dst: &mut [u8]) -> Result<usize, zip::ParsingError> {
        let len = cmp::min(self.buf.remaining(), dst.len());

        Buf::copy_to_slice(&mut self.buf, &mut dst[0..len]);
        Ok(len)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ZipFile {
    Deflated(Bytes),
    Uncompressed(Bytes),
}

#[derive(Debug, Snafu)]
#[snafu(display("{inner}"))]
struct DecompressError {
    inner: miniz_oxide::inflate::DecompressError,
}

/// Implementation of [ClassPathEntry] that searches for `.class` file inside a `.jar` file
pub struct JarFileClassPathEntry {
    file_name: String,
    filesystem: HashMap<String, RefCell<ZipFile>>,
}

impl Debug for JarFileClassPathEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "JarFileClassPathEntry {{ file_name: {} }}",
            self.file_name
        )
    }
}

impl JarFileClassPathEntry {
    pub fn new<P: AsRef<Path>>(fs: &dyn JvmIo, path: P) -> Result<Self, JarFileError> {
        let path = path.as_ref();
        if !fs.exists(path) {
            return NotFoundSnafu { path }.fail();
        }

        let file = fs
            .read(path)
            .map_err(|source| ReadingSnafu { path, source }.build())?;
        let mut data = ZipData {
            buf: Bytes::from(file),
        };
        let zip: ZipArchive<ZipData> = ZipArchive::new(&mut data);

        let mut filesystem = HashMap::new();
        for mut file in zip {
            let name = file
                .file_name()
                .context(InvalidFileNameSnafu { path })?
                .to_string();
            let mut buf = Vec::with_capacity(file.file_size().try_into().unwrap());
            file.read_exact(buf.as_mut_slice())
                .map_err(|source| InvalidJarSnafu { path, source }.build())?;
            let buf = Bytes::from(buf);
            let zip_file = match file.info.compression_method {
                CompressMethod::Uncompress => ZipFile::Uncompressed(buf),
                CompressMethod::Deflated => ZipFile::Deflated(buf),
                method => {
                    return UnsupportedCompressMethodSnafu {
                        jar: path,
                        file: name,
                        method,
                    }
                    .fail()
                }
            };
            filesystem.insert(name, RefCell::new(zip_file));
        }

        Ok(Self {
            file_name: path.to_string_lossy().to_string(),
            filesystem,
        })
    }
}

impl ClassPathEntry for JarFileClassPathEntry {
    fn resolve(
        &self,
        _fs: &dyn JvmIo,
        class_name: &str,
    ) -> Result<Option<Bytes>, ClassLoadingError> {
        let class_file_name = format!("{class_name}.class");
        return match self.filesystem.get(&class_file_name) {
            Some(zip_file_ref) => {
                let zip_file = zip_file_ref.clone().into_inner();
                let buf = match zip_file {
                    ZipFile::Deflated(buf) => {
                        let decompressed = decompress_to_vec(buf.as_ref())
                            .map_err(|inner| DecompressSnafu { inner }.build())
                            .map_err(ClassLoadingError::new)?;
                        Bytes::from(decompressed)
                    }
                    ZipFile::Uncompressed(buf) => buf,
                };
                *zip_file_ref.borrow_mut() = ZipFile::Uncompressed(buf.clone());
                Ok(Some(buf))
            }
            None => Ok(None),
        };
    }
}

/// Error returned if searching a class inside a Jar fails
#[derive(Snafu, Debug)]
pub enum JarFileError {
    /// The jar file does not exist!
    #[snafu(display("file {path:?} not found"))]
    NotFound { path: PathBuf },

    /// Generic I/O error reading the file
    #[snafu(display("error reading file {path:?}"))]
    ReadingError {
        path: PathBuf,
        #[snafu(source(false))]
        source: no_std_io::io::Error,
    },

    /// Jar contains invalid UTF-8 file name
    #[snafu(display("jar {path:?} contains an invalid file name: {source}"))]
    InvalidFileName { path: PathBuf, source: Utf8Error },

    /// The file is not actually a valid jar
    #[snafu(display("file {path:?} is not a valid jar"))]
    InvalidJar {
        path: PathBuf,
        #[snafu(source(false))]
        source: ParsingError,
    },

    /// A file's compression method is not supported
    #[snafu(display("file {file:?} in jar {jar:?} uses unsupported compression method `{method:?}` ({})", *method as u8))]
    UnsupportedCompressMethod {
        file: PathBuf,
        jar: PathBuf,
        method: CompressMethod,
    },
}

#[cfg(all(test, feature = "std"))]
mod tests {

    use unix_path::PathBuf;

    use crate::{
        class_path_entry::tests::{assert_can_find_class, assert_cannot_find_class},
        io::StdJvmIo,
        jar_file_class_path_entry::{JarFileClassPathEntry, JarFileError},
    };

    #[test]
    fn jar_file_not_found() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources/not_found.jar");
        let entry = JarFileClassPathEntry::new(&StdJvmIo, path.clone());
        assert!(matches!(
            entry.expect_err("should have thrown an error"),
            JarFileError::NotFound { path: p } if p == path,

        ));
    }

    #[test]
    fn file_is_not_a_jar() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources/compile.sh");

        let entry = JarFileClassPathEntry::new(&StdJvmIo, path.clone());
        assert!(matches!(
            entry.expect_err("should have thrown an error"),
            JarFileError::InvalidJar { path: p, .. } if p == path,
        ));
    }

    #[test]
    fn valid_jar_file_can_search_for_class_file() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources/sample.jar");
        let entry =
            JarFileClassPathEntry::new(&StdJvmIo, path).expect("should have read the jar file");

        assert_can_find_class(&entry, &StdJvmIo, "rjvm/NumericTypes");
        assert_can_find_class(&entry, &StdJvmIo, "rjvm/ControlFlow");
        assert_cannot_find_class(&entry, &StdJvmIo, "rjvm/Foo");
    }
}
