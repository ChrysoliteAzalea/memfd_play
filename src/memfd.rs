use crate::check_error;
use std::os::fd::{OwnedFd, FromRawFd, BorrowedFd, AsRawFd};
use std::ffi::CStr;

#[allow(dead_code)]
pub const F_SEAL_SEAL: libc::c_int = libc::F_SEAL_SEAL;
pub const F_SEAL_SHRINK: libc::c_int = libc::F_SEAL_SHRINK;
pub const F_SEAL_GROW: libc::c_int = libc::F_SEAL_GROW;
pub const F_SEAL_WRITE: libc::c_int = libc::F_SEAL_WRITE;

#[inline(always)]
pub fn create(name: &CStr) -> std::io::Result<OwnedFd>
{
   let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING) };
   check_error!(fd >= 0);
   Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

#[inline(always)]
pub fn seal(fd: BorrowedFd, seal: libc::c_int) -> std::io::Result<()>
{
   let r = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_ADD_SEALS, seal) };
   check_error!(r >= 0);
   Ok(())
}

#[inline(always)]
#[allow(dead_code)]
pub fn get_seals(fd: BorrowedFd) -> std::io::Result<()>
{
   let r = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GET_SEALS) };
   check_error!(r >= 0);
   Ok(())
}