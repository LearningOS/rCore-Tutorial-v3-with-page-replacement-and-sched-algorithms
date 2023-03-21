use alloc::collections::{BTreeMap, VecDeque};
use rand_chacha::rand_core::block;
use core::marker::Copy;
use core::cmp::{Ord};
use crate::Manage;
use crate::Schedule;
use crate::syscall_args::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;


struct LotteryBlock {
    priority: usize,
    is_ready: bool
}

pub struct LotteryManager<T, I: Copy + Ord> {
    tasks: BTreeMap<I, T>,
    info_map: BTreeMap<I, LotteryBlock>,
    total_count: usize,
    rng: ChaCha20Rng
}

impl<T, I: Copy + Ord> LotteryManager<T, I> {
    pub fn new() -> Self {
        Self { 
            tasks: BTreeMap::new(), 
            info_map: BTreeMap::new(), 
            total_count: 0,
            rng: ChaCha20Rng::seed_from_u64(233u64)
        }
    }
}

impl<T, I: Copy + Ord> Manage<T, I> for LotteryManager<T, I> {
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
        let block = self.info_map.remove(&id);
        if let Some(b) = block {
            if b.is_ready {
                self.total_count -= b.priority;
            }
        }
    }
}

impl<T, I: Copy + Ord> Schedule<I> for LotteryManager<T, I> {
    fn add(&mut self, id: I) {
        let block = self.info_map.get_mut(&id);
        let add = if let Some(t) = block {
            t.is_ready = true;
            t.priority
        } else {
            self.info_map.insert(id, LotteryBlock { priority: 16, is_ready: true });
            16
        };
        self.total_count += add;
    }

    fn fetch(&mut self) -> Option<I> {
        let lucky_dog = self.rng.gen_range(0..self.total_count);
        let mut sum = 0;
        for (&id, block) in self.info_map.iter() {
            sum += block.priority;
            if sum > lucky_dog{
                return Some(id);
            }
        }
        None
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        let block = self.info_map.get_mut(&id).unwrap();
        if block.priority > args.priority {
            self.total_count -= block.priority - args.priority;
        } else {
            self.total_count += args.priority - block.priority;
        }
        block.priority = args.priority;
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let block = self.info_map.get(&parent_id).unwrap();
        self.info_map.insert(child_id, LotteryBlock { priority: block.priority, is_ready: false });
    }

    fn update_sched_to(&mut self, id: I, time: usize) {}
    fn update_suspend(&mut self, id: I, time: usize) {
        let block = self.info_map.get_mut(&id).unwrap();
        block.is_ready = false;
        self.total_count -= block.priority;
    }
    fn update_sleep(&mut self, id: I) {}
}