use alloc::collections::{BTreeMap, VecDeque};
use super::Manage;
use super::Schedule;
use core::marker::Copy;
use core::cmp::Ord;
use core::option::Option;
use crate::syscall_args::*;

pub struct DefaultManager<T, I: Copy + Ord> {
    tasks: BTreeMap<I, T>,
    ready_queue: VecDeque<I>,
}

impl<T, I: Copy + Ord> DefaultManager<T, I> {
    /// 新建任务管理器
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            ready_queue: VecDeque::new(),
        }
    }
}

impl<T, I: Copy + Ord> Manage<T, I> for DefaultManager<T, I> {
    /// 插入一个新任务
    #[inline]
    fn insert(&mut self, id: I, task: T) {
        self.tasks.insert(id, task);
    }
    /// 根据 id 获取对应的任务
    #[inline]
    fn get_mut(&mut self, id: I) -> Option<&mut T> {
        self.tasks.get_mut(&id)
    }
    /// 删除任务实体
    #[inline]
    fn delete(&mut self, id: I) {
        self.tasks.remove(&id);
    }
}

impl<T, I: Copy + Ord> Schedule<I> for DefaultManager<T, I> {
    /// 添加 id 进入调度队列
    fn add(&mut self, id: I) {
        self.ready_queue.push_back(id);
    }
    /// 从调度队列中取出 id
    fn fetch(&mut self) -> Option<I> {
        self.ready_queue.pop_front()
    }
    
    fn update_exec(&mut self, id: I, args: &ExecArgs) {}

    fn update_fork(&mut self, parent_id: I, child_id: I) {}

    fn update_sched_to(&mut self, id: I, time: usize) {}

    fn update_suspend(&mut self, id: I, time: usize) {}

    fn update_sleep(&mut self, id: I) {}
}