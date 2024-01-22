use std::env;
use std::fs::File;

use std::io::prelude::*;

extern crate memfd;

use std::os::fd::RawFd;
use std::os::fd::IntoRawFd;
extern crate nix;

use cstr::cstr;


fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2
	{
		println!("Not enough arguments given.");
		std::process::exit(2);
	}
	let ref a1 = &args[1];
	let mut file = match File::open::<&String>(a1) {
        Err(why) => panic!("couldn't open {}: {}", args[1], why),
        Ok(file) => file,
    };
    let opts = memfd::MemfdOptions::default().allow_sealing(true).close_on_exec(false);
    let mfd = opts.create("");
    let seal_shrink = mfd.as_ref().expect("REASON").add_seals(&[
        memfd::FileSeal::SealShrink
    ]);
    match seal_shrink {
    	Ok(seal_shrink) => seal_shrink,
    	Err(why) => panic!("couldn't add F_SEAL_SHRINK to the virtual file: {}", why),
    };
    let mut s: usize = 1;
    let mut readbuf = [0; 2049];
    //let mut R: std::result::Result<T, E>;
    while s > 0 {
    	s = file.read(&mut readbuf[..]).unwrap();
    	let mut _wr_s = mfd.as_ref().expect("REASON").as_file().write(&readbuf[..]);
    }
	let seal_grow_write = mfd.as_ref().expect("REASON").add_seals(&[
		memfd::FileSeal::SealGrow,
		memfd::FileSeal::SealWrite,
	]);
	match seal_grow_write {
		Ok(seal_grow_write) => seal_grow_write,
		Err(why) => panic!("couldn't add F_SEAL_GROW and F_SEAL_WRITE to the virtual file: {}", why),
	};
	let fd: RawFd = mfd.expect("REASON").into_raw_fd();
	let _f_close = match nix::unistd::close(0) {
		Ok(f_close) => f_close,
		Err(why) => panic!("couldn't close the default stdin: {}", why),
	};
	let _f_dup = match nix::unistd::dup2(fd, 0) {
		Ok(f_dup) => f_dup,
		Err(why) => panic!("couldn't set the virtual file as stdin: {}", why),
	};
	let _d_close = match nix::unistd::close(fd) {
		Ok(d_close) => d_close,
		Err(why) => panic!("couldn't close the fd: {}", why),
	};
	let _excv = nix::unistd::execv(cstr!("/usr/bin/mpv"), &[cstr!("mpv"), cstr!("-")]);
	panic!("execve has failed");
}
