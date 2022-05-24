pub mod util;

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use win::util::*;
#[cfg(target_os = "windows")]
pub use win::sys_browsers;
