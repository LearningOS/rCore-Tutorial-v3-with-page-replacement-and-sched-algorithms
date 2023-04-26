use core::marker::PhantomData;
use core::option::Option;

use crate::{ACCESS_FLAG, PFF_T};
use crate::plugins::{Manage, handle_global_pagefault};
use crate::frame_allocator::{FrameTracker, frame_check};
use alloc::vec::Vec;
use kernel_vm::{AddressSpace, VmMeta, PageManager, VPN, Pte, PPN};
use alloc::collections::VecDeque;

use rcore_utils::get_time;

pub struct PffManager<Meta: VmMeta, M: PageManager<Meta> + 'static> {
    queue: VecDeque<((PPN<Meta>, VPN<Meta>, usize), FrameTracker)>,
    last_pgfault: usize, // timestamp
    dummy: PhantomData<M>
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> PffManager<Meta, M> {
    fn get_pte(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) -> Option<Pte<Meta>> {
        memory_set.translate_to_pte(vpn.base())
    }

    fn has_accessed(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) -> bool {
        let pte = Self::get_pte(memory_set, vpn).unwrap();
        let flags = pte.flags();
        (flags.val() & ACCESS_FLAG) != 0 
    }

    fn clear_accessed(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) {
        memory_set.clear_accessed(*vpn);
    }

    fn pop_unaccessed_frames<F>(&mut self, get_memory_set: &F) -> Vec<(PPN<Meta>, VPN<Meta>, usize)>
        where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        let ids: Vec<usize> = self.queue.iter().enumerate()
            .filter(|(id, (info, frame))| !Self::has_accessed(get_memory_set(info.2), &info.1))
            .map(|(id, (info, frame))| id).collect();

        ids.iter().rev().map(|&id| self.queue.remove(id).unwrap().0).collect()
    }
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> Clone for PffManager<Meta, M> {
    fn clone(&self) -> Self {
        unimplemented!("shouldn't clone global manager");
    }
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> Manage<Meta, M> for PffManager<Meta, M> {
    fn new() -> Self {
        Self { queue: VecDeque::new(), last_pgfault: usize::MAX, dummy: PhantomData }
    }

    fn handle_pagefault<F>(&mut self, get_memory_set: &F, vpn: VPN<Meta>, task_id: usize)
            where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        handle_global_pagefault(get_memory_set, vpn, task_id, self);
    }

    fn insert_frame(&mut self, vpn: VPN<Meta>, frame: FrameTracker) {
        unimplemented!("this should not be called for global manager");
    }

    fn get_next_frame(&mut self, memory_set: &mut AddressSpace<Meta, M>) -> Option<VPN<Meta>> {
        unimplemented!("this should not be called for global manager");
    }

    fn insert_global_frame(&mut self, vpn: VPN<Meta>, ppn: PPN<Meta>, task_id: usize, frame: FrameTracker) {
        self.queue.push_back(((ppn, vpn, task_id), frame));
    }

    fn work<F>(&mut self, get_memory_set: &F) -> Vec<(PPN<Meta>, VPN<Meta>, usize)> 
        where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
            if self.last_pgfault == usize::MAX {
                self.last_pgfault = get_time();
            }
            let cur_time = get_time();
            if cur_time - self.last_pgfault > PFF_T {
                let mut ret = self.pop_unaccessed_frames(get_memory_set);
                if !frame_check() && ret.len() == 0 {
                    ret.push(self.queue.pop_front().unwrap().0);
                }
                ret
            } else {
                if !frame_check() {
                    let mut ret = self.pop_unaccessed_frames(get_memory_set);
                    if ret.len() == 0 {
                        ret.push(self.queue.pop_front().unwrap().0);
                    }
                    ret
                } else {
                    for ((ppn, vpn, task_id), _) in self.queue.iter() {
                        let _mem_set = get_memory_set(*task_id);
                        _mem_set.clear_accessed(*vpn);
                    }
                    Vec::new()
                }
            }
    }

    fn clear_frames(&mut self, task_id: usize) {
        let ids: Vec<usize> = self.queue.iter().enumerate()
            .filter(|(_, (info, _))| info.2 == task_id)
            .map(|(id, _)| id).collect();
        ids.iter().rev().for_each(|&id| { self.queue.remove(id); });
    }
}