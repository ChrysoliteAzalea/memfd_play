use crate::check_error;
use std::os::fd::{BorrowedFd, AsRawFd};

pub type OffsetType = libc::off_t;

#[inline(always)]
pub fn sendfile(out_fd: BorrowedFd, in_fd: BorrowedFd, offset: Option<&mut OffsetType>, size: usize) -> std::io::Result<usize>
{
   let r: isize = unsafe { libc::sendfile(out_fd.as_raw_fd(), in_fd.as_raw_fd(), offset.map(|r| r as *mut OffsetType).unwrap_or(core::ptr::null_mut::<OffsetType>()), size) };
   check_error!(r >= 0);
   Ok(r as usize)
}

#[inline(always)]
pub fn copy_file_range(fd_in: BorrowedFd, off_in: Option<&mut OffsetType>, fd_out: BorrowedFd, off_out: Option<&mut OffsetType>, len: usize) -> std::io::Result<usize>
{
   let r: isize = unsafe { libc::copy_file_range(fd_in.as_raw_fd(), off_in.map(|r| r as *mut OffsetType).unwrap_or(core::ptr::null_mut::<OffsetType>()), fd_out.as_raw_fd(), off_out.map(|r| r as *mut OffsetType).unwrap_or(core::ptr::null_mut::<OffsetType>()), len, 0) };
   check_error!(r >= 0);
   Ok(r as usize)
}

#[inline(always)]
pub fn splice(fd_in: BorrowedFd, off_in: Option<&mut OffsetType>, fd_out: BorrowedFd, off_out: Option<&mut OffsetType>, len: usize) -> std::io::Result<usize>
{
   let r: isize = unsafe { libc::splice(fd_in.as_raw_fd(), off_in.map(|r| r as *mut OffsetType).unwrap_or(core::ptr::null_mut::<OffsetType>()), fd_out.as_raw_fd(), off_out.map(|r| r as *mut OffsetType).unwrap_or(core::ptr::null_mut::<OffsetType>()), len, 0) };
   check_error!(r >= 0);
   Ok(r as usize)
}