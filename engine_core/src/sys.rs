#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub mod platform {
    #[cfg(unix)]
    pub use super::unix::*;
    #[cfg(windows)]
    pub use super::windows::*;
}
