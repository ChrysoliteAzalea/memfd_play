#![feature(maybe_uninit_fill)]
#![feature(never_type)]
mod handle_syscall_error;
mod memfd;
mod copy;
mod exec_trait;

use std::os::fd::{BorrowedFd, AsFd, AsRawFd};
use crate::exec_trait::Executable;
use std::io::{Write, Seek, SeekFrom};
use std::env::VarError;
use std::ffi::OsString;

fn cfr_transfer(src: BorrowedFd, dest: BorrowedFd, rate: usize) -> std::io::Result<()>
{
   let mut src_offset: copy::OffsetType = 0;
   let mut dest_offset: copy::OffsetType = 0;
   let mut r: usize = 1;
   while r > 0
   {
      r = copy::copy_file_range(src, Some(&mut src_offset), dest, Some(&mut dest_offset), rate)?;
   }
   Ok(())
}

fn sf_transfer(src: BorrowedFd, dest: BorrowedFd, rate: usize) -> std::io::Result<()>
{
   let mut src_offset: copy::OffsetType = 0;
   let mut r: usize = 1;
   while r > 0
   {
      r = copy::sendfile(dest, src, Some(&mut src_offset), rate)?;
   }
   Ok(())
}

fn spl_transfer(src: BorrowedFd, dest: BorrowedFd, rate: usize) -> std::io::Result<()>
{
   let mut dest_offset: copy::OffsetType = 0;
   let mut r: usize = 1;
   while r > 0
   {
      r = copy::splice(src, None, dest, Some(&mut dest_offset), rate)?;
   }
   Ok(())
}

fn optimized_transfer(src: BorrowedFd, dest: BorrowedFd, rate: usize) -> std::io::Result<()>
{
   if let Ok(()) = spl_transfer(src, dest, rate)
   {
      return Ok(());
   }
   if let Ok(()) = cfr_transfer(src, dest, rate)
   {
      return Ok(());
   }
   sf_transfer(src, dest, rate)
}

fn ordinary_transfer<R: std::io::Read, W: std::io::Write>(src: &mut R, dest: &mut W, rate: usize) -> std::io::Result<()>
{
   let mut buf: Vec<u8> = Vec::new();
   buf.try_reserve_exact(rate)?;
   buf.spare_capacity_mut().write_filled(0);
   unsafe { buf.set_len(buf.capacity()) }; // SAFETY: initialized earlier
   let mut r: usize = 1;
   while r > 0
   {
      r = src.read(&mut buf)?;
      dest.write(&mut buf[0..r])?;
   }
   dest.flush()?;
   Ok(())
}

#[inline(always)]
fn make_stdin(fd: BorrowedFd) -> std::io::Result<()>
{
   let fd = unsafe { libc::dup2(fd.as_raw_fd(), 0) };
   check_error!(fd == 0);
   Ok(())
}

fn trim_arg0(original: &str) -> &str
{
   let mut iter = original.rsplit('/');
   iter.next().unwrap_or(original)
}

fn handle_error(action: &str, failure: &dyn core::error::Error) -> !
{
   eprintln!("An error has occurred while {action}: {failure}");
   std::process::exit(1)
}

fn main()
{
   let mut args = std::env::args_os();
   let p_name = args.next().map(OsString::into_string).map(Result::ok).flatten().unwrap();
   let mut filename = args.next();
   if filename == Some("-h".into()) || filename == Some("--help".into())
   {
      println!("Usage: {0} [filename]
      
If [filename] is not set, stdin is used

{0} can be configured with environment variables:

CUSTOM_MPV_OPTIONS -- options to pass to mpv
MEMFD_COPY_RATE -- data transfer rate
CUSTOM_PLAYER_BINARY -- invoke custom player (not mpv)", trim_arg0(&p_name));
      return ();
   }
   if filename == Some("-v".into()) || filename == Some("--version".into())
   {
      println!("{} {}", trim_arg0(&p_name), env!("CARGO_PKG_VERSION"));
      return ();
   }
   if filename == Some("--".into())
   {
      filename = args.next();
   }
   let opts: Option<String> = match std::env::var("CUSTOM_MPV_OPTIONS")
   {
      Ok(o) => Some(o),
      Err(failure) => { match failure {
         VarError::NotPresent => None,
         VarError::NotUnicode(s) => {
            let mut handle = std::io::stderr().lock();
            handle.write("An error has occurred: CUSTOM_MPV_OPTIONS environment variable value ".as_bytes()).unwrap();
            handle.write(&s.into_encoded_bytes()).unwrap();
            handle.write(b" is not a valid UTF-8 string. ").unwrap();
            handle.write(p_name.as_bytes()).unwrap();
            handle.write(b" performs string manipulations on it, which requires it to be an UTF-8 string").unwrap();
            handle.flush().unwrap();
            core::mem::drop(handle);
            std::process::exit(1)
         },
      } },
   };
   let copyrate = std::env::var("MEMFD_COPY_RATE").ok().map(|s| usize::from_str_radix(&s, 10).ok()).flatten().unwrap_or(4096);
   let player = std::env::var_os("CUSTOM_PLAYER_BINARY");
   let mut memfd = memfd::create(c"").map(std::fs::File::from).unwrap_or_else(|f| handle_error("creating a memfd instance", &f));
   memfd::seal(memfd.as_fd(), memfd::F_SEAL_SHRINK).unwrap_or_else(|f| handle_error("sealing the memfd instance against shrinking", &f));
   match filename
   {
      Some(f) => {
         let mut file = std::fs::File::open(&f).unwrap_or_else(|failure| handle_error(format!("opening {} for reading", f.clone().into_string().unwrap_or_else(|_| "(not UTF-8)".to_owned())).as_str(), &failure));
         if optimized_transfer(file.as_fd(), memfd.as_fd(), copyrate).is_err()
         {
            let _ = file.seek(SeekFrom::Start(0));
            ordinary_transfer(&mut file, &mut memfd, copyrate).unwrap_or_else(|failure| handle_error(format!("transferring data from {} to memory", f.into_string().unwrap_or_else(|_| "(not UTF-8)".to_owned())).as_str(), &failure));
         }
      },
      None => {
         if optimized_transfer(std::io::stdin().as_fd(), memfd.as_fd(), copyrate).is_err()
         {
            ordinary_transfer(&mut std::io::stdin().lock(), &mut memfd, copyrate).unwrap_or_else(|failure| handle_error("transferring data from stdin to memory", &failure));
         }
      },
   }
   let mut cmd = std::process::Command::new(player.unwrap_or("mpv".into()));
   if let Some(list) = opts
   {
      let mut escaped = list.replace("\\\\", "{ESCAPEDBACKSLASH}");
      escaped = escaped.replace("\\ ", "{ESCAPEDSPACE}");
      for i in escaped.split(' ')
      {
         let mut unescaped = i.replace("{ESCAPEDSPACE}", " ");
         unescaped = unescaped.replace("{ESCAPEDBACKSLASH}", "\\");
         cmd.arg(&unescaped);
      }
   }
   cmd.arg("-");
   cmd.env_remove("CUSTOM_MPV_OPTIONS").env_remove("MEMFD_COPY_RATE").env_remove("CUSTOM_PLAYER_BINARY");
   memfd::seal(memfd.as_fd(), memfd::F_SEAL_GROW | memfd::F_SEAL_WRITE).unwrap_or_else(|f| handle_error("sealing the memfd instance against growing or modification", &f));
   make_stdin(memfd.as_fd()).unwrap_or_else(|f| handle_error("redirecting stdin", &f));
   cmd.perform_exec().unwrap_or_else(|f| handle_error("executing the media player", &f));
}