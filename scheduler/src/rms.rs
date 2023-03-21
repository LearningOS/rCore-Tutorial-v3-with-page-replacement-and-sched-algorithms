use core::marker::Copy;
use core::cmp::{Ord, Ordering};
use core::clone::Clone;
use alloc::collections::{BTreeMap, BinaryHeap, VecDeque};
use crate::Manage;
use crate::Schedule;
use crate::syscall_args::*;

#[derive(Clone, Copy)]
struct RMSBlock<I: Copy + Ord> {
    task_id: I,
    period: isize
}

impl<I: Copy + Ord> PartialOrd for RMSBlock<I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.period.partial_cmp(&self.period)
    }
}

impl<I: Copy + Ord> PartialEq for RMSBlock<I> {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
    }
}

impl<I: Copy + Ord> Ord for RMSBlock<I> {
    fn cmp(&self, other:&Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<I: Copy + Ord> Eq for RMSBlock<I> {}

pub struct RMSManager<T, I: Copy + Ord> {
    tasks: BTreeMap<I, T>,
    period_map: BTreeMap<I, isize>,
    heap: BinaryHeap<RMSBlock<I>>
}


impl<T, I: Copy + Ord> RMSManager<T, I> {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            period_map: BTreeMap::new(),
            heap: BinaryHeap::new()
        }
    }

    pub fn get_list(&self) -> VecDeque<(I, isize)> {
        self.heap.iter().map(|block| (block.task_id, block.period)).collect()
    }
}

impl<T, I: Copy + Ord> Manage<T, I> for RMSManager<T, I> {
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
        self.period_map.remove(&id);
    }
}

impl<T, I: Copy + Ord> Schedule<I> for RMSManager<T, I> {
    fn add(&mut self, id: I) {
        let period = match self.period_map.get(&id) {
            None => {
                self.period_map.insert(id.clone(), isize::MAX);
                isize::MAX
            },
            Some(t) => *t
        };
        self.heap.push(RMSBlock { task_id: id, period: period });
    }

    fn fetch(&mut self) -> Option<I> {
        match self.heap.pop() {
            None => None,
            Some(t) => Some(t.task_id)
        }
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        self.period_map.insert(id, args.period);
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let parent_time = self.period_map.get(&parent_id).unwrap();
        self.period_map.insert(child_id, *parent_time);
    }

    fn update_sched_to(&mut self, id: I, time: usize) {}
    fn update_suspend(&mut self, id: I, time: usize) {}
    fn update_sleep(&mut self, id: I) {}
}