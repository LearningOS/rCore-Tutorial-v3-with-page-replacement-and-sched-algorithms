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

#[cfg(feature = "lottery")]
pub struct ExecArgs {
    pub priority: usize
}

#[cfg(feature = "edf")]
pub struct ExecArgs {
    pub period: isize,
    pub init_ddl: isize
}

#[cfg(feature = "rms")]
pub struct ExecArgs {
    pub period: isize
}

#[cfg(feature = "mlfq")] 
pub struct ExecArgs;