use alloc::collections::BTreeMap;
use alloc::vec;
use lazy_static::lazy_static;

pub static SJF_TIME: usize = 1;
pub static STRIDE_PRIORITY: usize = 16;
pub static LOTTERY_PRIORITY: usize = 16;
pub static EDF_PERIOD: isize = -1;
pub static EDF_INIT_DDL: isize = 0;
pub static EDF_ARGS: (isize, isize) = (EDF_PERIOD, EDF_INIT_DDL);
pub static RMS_PERIOD: isize = -1;

lazy_static! {
    pub static ref INIT_PROC_AND_ARGS: BTreeMap<&'static str, usize> = vec![
        ("sjftests", &SJF_TIME as *const _ as usize),
        ("stridetests", &STRIDE_PRIORITY as *const _ as usize),
        ("lotterytests", &LOTTERY_PRIORITY as *const _ as usize),
        ("edftests", &EDF_ARGS as *const _ as usize),
        ("rmstests", &RMS_PERIOD as *const _ as usize),
        ("mlfqtests", &() as *const _ as usize),
        ("user_shell", &() as *const _ as usize)
    ].into_iter().collect();
}