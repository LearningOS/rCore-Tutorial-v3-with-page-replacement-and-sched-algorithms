use core::marker::Copy;
use core::cmp::{Ord};
use alloc::collections::{VecDeque, LinkedList, BTreeMap};
use crate::Manage;
use crate::Schedule;
use crate::syscall_args::*;

const PRIORITY_NUM: usize = 16;
const MIN_PRIOR: usize = PRIORITY_NUM - 1;

pub struct MLFQManager<T, I: Copy + Ord> {
    tasks: BTreeMap<I, T>,
    priority_map: BTreeMap<I, usize>, // 0 for max priority, 15 min
    queues: VecDeque<LinkedList<I>>,
    cur_giveup: bool
}

impl<T, I: Copy + Ord> MLFQManager<T, I> {
    pub fn new() -> Self{
        let mut ret = Self {
            tasks: BTreeMap::new(),
            priority_map: BTreeMap::new(),
            queues: VecDeque::new(),
            cur_giveup: false
        };
        for _ in 0..PRIORITY_NUM {
            ret.queues.push_back(LinkedList::new());
        } 
        ret
    }

    pub fn get_priority(&self, id: &I) -> usize {
        *self.priority_map.get(id).unwrap()
    }
}

impl<T, I: Copy + Ord> Manage<T, I> for MLFQManager<T, I> {
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
        self.priority_map.remove(&id);
    }
}

impl<T, I: Copy + Ord> Schedule<I> for MLFQManager<T, I> {
    fn add(&mut self, id: I) {
        let priority = match self.priority_map.get(&id) {
            None => {
                self.priority_map.insert(id.clone(), 0);
                0
            },
            Some(p) => *p
        };
        self.queues[priority].push_back(id);
    }

    fn fetch(&mut self) -> Option<I> {
        for list in self.queues.iter_mut() {
            if list.len() != 0 {
                return list.pop_front();
            }
        }
        None
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        self.priority_map.insert(id.clone(), 0); // arg is empty
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let p_prior = self.priority_map.get(&parent_id).unwrap();
        self.priority_map.insert(child_id, *p_prior);
    }

    fn update_sched_to(&mut self, id: I, time: usize) {
        self.cur_giveup = false;
    }
    
    fn update_suspend(&mut self, id: I, time: usize) {
        if !self.cur_giveup {
            let priority = self.priority_map.get(&id).unwrap();
            let new_priority = if *priority != MIN_PRIOR { *priority + 1 } else { MIN_PRIOR };
            self.priority_map.insert(id, new_priority);
        }
    }

    fn update_sleep(&mut self, id: I) {
        self.cur_giveup = true;
    }
}