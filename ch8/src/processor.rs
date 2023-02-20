use crate::process::{Process, Thread};
use alloc::collections::{BTreeMap, VecDeque};
use rcore_task_manage::{PThreadManager, ProcId, ThreadId, Manage};
pub use rcore_task_manage::Manager as ThreadManager;

pub static mut PROCESSOR: PThreadManager<Process, Thread, ThreadManager<Thread, ThreadId>, ProcManager> =
    PThreadManager::new();

/// 进程管理器
/// `procs` 中保存所有的进程实体
pub struct ProcManager {
    procs: BTreeMap<ProcId, Process>,
}

impl ProcManager {
    /// 新建进程管理器
    pub fn new() -> Self {
        Self {
            procs: BTreeMap::new(),
        }
    }
}

impl Manage<Process, ProcId> for ProcManager {
    /// 插入一个新任务
    #[inline]
    fn insert(&mut self, id: ProcId, item: Process) {
        self.procs.insert(id, item);
    }
    /// 根据 id 获取对应的任务
    #[inline]
    fn get_mut(&mut self, id: ProcId) -> Option<&mut Process> {
        self.procs.get_mut(&id)
    }
    /// 删除任务实体
    #[inline]
    fn delete(&mut self, id: ProcId) {
        self.procs.remove(&id);
    }
}
