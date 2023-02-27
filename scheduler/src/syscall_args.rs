use core::ptr::NonNull;

#[cfg(feature = "seq")]
pub struct ExecArgs;

#[cfg(feature = "sjf")]
pub struct ExecArgs {
    pub time: usize
}