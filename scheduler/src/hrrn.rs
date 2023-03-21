use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use core::marker::Copy;
use core::cmp::{Ord, Ordering};
use crate::Manage;
use crate::Schedule;
use crate::syscall_args::*;
use rcore_utils::get_time_ms;

struct HRRNTaskBlock<I: Copy + Ord> {
    task_id: I,
    time_total: usize,
    time_wait: usize,
    last_stop_time: usize,
    is_ready: bool
}

impl<I: Copy + Ord> HRRNTaskBlock<I> {
    // pub fn compute_rr(&self, cur_time: usize) -> f64 {
    //     let t = self.time_total;
    //     let mut w = self.time_wait;
    //     if self.last_stop_time != usize::MAX {
    //         w += cur_time - self.last_stop_time;
    //     }
    //     1.0 + (w as f64 / t as f64)
    // }

    pub fn cmp(&self, other: &Self, cur_time: usize) -> Ordering {
        let ts = self.time_total;
        let ws = if self.last_stop_time != usize::MAX { (cur_time - self.last_stop_time) + self.time_wait } else { self.time_wait };
        let to = other.time_total;
        let wo = if other.last_stop_time != usize::MAX { (cur_time - other.last_stop_time) + other.time_wait } else { other.time_wait };
        (ws * to).cmp(&(wo * ts))
    }

    pub fn update_wait_time(&mut self, cur_time: usize) {
        if self.last_stop_time != usize::MAX {
            self.time_wait += cur_time - self.last_stop_time;
        }
    }
}

pub struct HRRNManager<T, I: Copy + Ord> {
    tasks: BTreeMap<I, T>,
    info_map: BTreeMap<I, HRRNTaskBlock<I>>,
    current: Option<I> // (id, st_time)
}

impl<T, I: Copy + Ord> HRRNManager<T, I> {
    pub fn new() -> Self {
        Self { 
            tasks: BTreeMap::new(),
            info_map: BTreeMap::new(),
            current: None
        }
    }
}

impl<T, I: Copy + Ord> HRRNManager<T, I> {
    pub fn get_list(&self) -> VecDeque<(I, usize, usize)> {
        let mut ret: VecDeque<_> = VecDeque::new();
        for (id, block) in self.info_map.iter() {
            ret.push_back((id.clone(), block.time_total, block.time_wait));
        }
        ret
    }
}

impl<T, I: Copy + Ord> Manage<T, I> for HRRNManager<T, I> {
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
        self.info_map.remove(&id);
        if let Some(cur_id) = self.current {
            if cur_id == id {
                self.current = None
            }
        }
    }
}

impl<T, I: Copy + Ord> Schedule<I> for HRRNManager<T, I> {
    fn add(&mut self, id: I) {
        if let Some(block) = self.info_map.get_mut(&id) {
            block.is_ready = true;
        } else {
            self.info_map.insert(id, HRRNTaskBlock {
                task_id: id,
                time_total: 1000_000_000,
                time_wait: 0,
                last_stop_time: usize::MAX,
                is_ready: true
            });
        }
    }

    fn fetch(&mut self) -> Option<I> {
        let time = get_time_ms();
        let task_block = self.info_map.iter()
            .filter(|&(_, block)| block.is_ready)
            .max_by(|&a, &b| a.1.cmp(b.1, time));
            
        match task_block {
            None => None, 
            Some((id, _)) => Some(id.clone())
        }
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        let block = self.info_map.get_mut(&id).unwrap();
        block.time_total = args.total_time;
        block.time_wait = 0; 
        block.last_stop_time = usize::MAX;
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let parent = self.info_map.get(&parent_id).unwrap();
        self.info_map.insert(child_id, HRRNTaskBlock { 
            task_id: child_id, 
            time_total: parent.time_total,
            time_wait: parent.time_wait, 
            last_stop_time: parent.last_stop_time,
            is_ready: parent.is_ready
        });
    }

    fn update_sched_to(&mut self, id: I, time: usize) {
        if let None = self.current {
            self.current = Some(id);
            let block = self.info_map.get_mut(&id).unwrap();
            block.update_wait_time(time);
        } else {
            panic!("call sched while current is not suspended!")
        }
    }

    fn update_suspend(&mut self, id: I, time: usize) {
        if let Some(cur_id) = self.current {
            if cur_id != id {
                panic!("suspend wrong id? ");
            }

            self.current = None;
            
            let block = self.info_map.get_mut(&cur_id).unwrap();
            block.last_stop_time = time;
            block.is_ready = false;
        } else {
            panic!("call suspend but current is none! ")
        }
    }

    fn update_sleep(&mut self, id: I) {}
}