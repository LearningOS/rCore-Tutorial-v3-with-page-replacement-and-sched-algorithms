use core::ptr::NonNull;

#[cfg(feature = "seq")]
pub struct ExecArgs;

#[cfg(feature = "sjf")]
pub struct ExecArgs {
    pub time: usize
}

#[cfg(feature = "stcf")]
pub struct ExecArgs {
    pub total_time: isize
}