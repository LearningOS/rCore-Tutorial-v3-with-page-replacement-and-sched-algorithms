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

#[cfg(feature = "hrrn")]
pub struct ExecArgs {
    pub total_time: usize
}

#[cfg(feature = "stride")]
pub struct ExecArgs {
    pub priority: usize
}