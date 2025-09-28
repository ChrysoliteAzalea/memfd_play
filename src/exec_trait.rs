use std::os::unix::process::CommandExt;

pub trait Executable
{
   fn perform_exec(&mut self) -> std::io::Result<!>;
}

impl Executable for std::process::Command
{
   #[inline(always)]
   fn perform_exec(&mut self) -> std::io::Result<!>
   {
      Err(self.exec())
   }
}