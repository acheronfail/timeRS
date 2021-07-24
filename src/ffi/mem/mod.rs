#[cfg(target_os = "macos")]
#[path = "darwin.rs"]
mod mem;

#[cfg(not(target_os = "macos"))]
#[path = "linux.rs"]
mod mem;

pub use mem::*;

use anyhow::Result;
use crate::ffi::sysconf;

pub fn page_size() -> Result<u64> {
    sysconf(libc::_SC_PAGESIZE).map(|x| x as u64)
}

pub fn memory_total() -> Result<u64> {
    let n_pages = sysconf(libc::_SC_PHYS_PAGES).map(|x| x as u64)?;
    Ok(n_pages * page_size()?)
}