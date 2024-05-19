use alloc::vec::Vec;
use core::time::Duration;
#[cfg(feature = "std")]
use std::path::Path as StdPath;

use no_std_io::io;
use unix_path::Path;

pub trait JvmIo: Send + Sync {
    fn read(&self, path: &Path) -> Result<Vec<u8>, io::Error>;
    fn exists(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    fn duration_since_epoch(&self) -> Duration;
}

#[cfg(feature = "std")]
pub struct StdJvmIo;

#[cfg(feature = "std")]
impl JvmIo for StdJvmIo {
    fn read(&self, path: &Path) -> Result<Vec<u8>, io::Error> {
        std::fs::read(path.to_str().unwrap())
    }

    fn exists(&self, path: &Path) -> bool {
        let p: &StdPath = path.to_str().unwrap().as_ref();
        p.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        let p: &StdPath = path.to_str().unwrap().as_ref();
        p.is_dir()
    }

    fn duration_since_epoch(&self) -> Duration {
        let start = std::time::SystemTime::now();
        start
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards")
    }
}
