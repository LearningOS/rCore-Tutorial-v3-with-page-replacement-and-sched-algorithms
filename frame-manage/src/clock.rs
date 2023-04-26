use core::marker::PhantomData;
use core::option::Option;

use crate::ACCESS_FLAG;
use crate::plugins::{Manage, handle_local_pagefault};
use crate::frame_allocator::FrameTracker;
use alloc::vec::Vec;
use alloc::vec;
use kernel_vm::{AddressSpace, VmMeta, PageManager, VPN, Pte, PPN};
use alloc::collections::VecDeque;

pub struct ClockManager<Meta: VmMeta, M: PageManager<Meta>> {
    queue: VecDeque<(VPN<Meta>, FrameTracker)>,
    ptr: usize,
    task_id: usize,
    dummy: PhantomData<M>
}

impl<Meta: VmMeta, M: PageManager<Meta>> ClockManager<Meta, M> {
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
}

impl<Meta: VmMeta, M: PageManager<Meta>> Clone for ClockManager<Meta, M> {
    fn clone(&self) -> Self {
        Self { queue: self.queue.clone(), ptr: self.ptr, task_id: usize::MAX, dummy: PhantomData }
    }
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> Manage<Meta, M> for ClockManager<Meta, M> {
    fn new() -> Self {
        Self { queue: VecDeque::new(), ptr: 0, task_id: usize::MAX, dummy: PhantomData }
    }

    fn handle_pagefault<F>(&mut self, get_memory_set: &F, vpn: VPN<Meta>, task_id: usize)
            where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        handle_local_pagefault(get_memory_set, vpn, task_id, self)
    }

    fn insert_frame(&mut self, vpn: VPN<Meta>, ppn: PPN<Meta>, task_id: usize, frame: FrameTracker) {
        if self.task_id == usize::MAX {
            self.task_id = task_id;
        }
        assert!(self.task_id == task_id);

        self.queue.push_back((vpn, frame))
    }

    fn work<F>(&mut self, get_memory_set: &F) -> Vec<(PPN<Meta>, VPN<Meta>, usize)>  
        where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        let memory_set = get_memory_set(self.task_id);
        assert!(self.queue.len() != 0);
        loop {
            if self.ptr >= self.queue.len() {
                self.ptr = 0;
            }
            let (vpn, frame) = &self.queue[self.ptr];
            if Self::has_accessed(memory_set, vpn) {
                Self::clear_accessed(memory_set, vpn);
                self.ptr += 1;
            } else {
                let (vpn, _) = self.queue.remove(self.ptr).unwrap();
                return vec![(PPN::new(0), vpn, 0)];
            }
        }
    }

    fn clear_frames(&mut self, task_id: usize) {}
}