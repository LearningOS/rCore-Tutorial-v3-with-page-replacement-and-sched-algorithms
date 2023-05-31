use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::vec;
use kernel_vm::{AddressSpace, VmMeta, PageManager, VPN, Pte, PPN};

use crate::frame_allocator::FrameTracker;
use crate::{ACCESS_FLAG, DIRTY_FLAG};


pub struct ClockQueue<Meta: VmMeta> {
    pub inner: VecDeque<(PPN<Meta>, VPN<Meta>, FrameTracker)>,
    pub ptr: usize
}

impl<Meta: VmMeta> ClockQueue<Meta> {
    fn get_pte<M: PageManager<Meta>>(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) -> Option<Pte<Meta>> {
        memory_set.translate_to_pte(vpn.base())
    }

    fn has_accessed<M: PageManager<Meta>>(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) -> bool {
        let pte = Self::get_pte(memory_set, vpn).unwrap();
        let flags = pte.flags();
        (flags.val() & ACCESS_FLAG) != 0 
    }

    fn clear_accessed<M: PageManager<Meta>>(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) {
        memory_set.clear_accessed(*vpn);
    }

    fn get_accessed_dirty<M: PageManager<Meta>>(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) -> (bool, bool) {
        let pte = Self::get_pte(memory_set, vpn).unwrap();
        let flags = pte.flags().val();
        ((flags & ACCESS_FLAG) != 0, (flags & DIRTY_FLAG) != 0)
    }
}

impl<Meta: VmMeta> ClockQueue<Meta> {
    pub fn new() -> Self {
        Self { inner: VecDeque::new(), ptr: 0 }
    }

    pub fn push_back(&mut self, item: (PPN<Meta>, VPN<Meta>, FrameTracker)) {
        self.inner.push_back(item);
    }

    pub fn len(&self) -> usize{
        self.inner.len()
    }

    pub fn work<M: PageManager<Meta>>(&mut self, memory_set: &mut AddressSpace<Meta, M>) -> (PPN<Meta>, VPN<Meta>) {
        loop {
            if self.ptr >= self.inner.len() {
                self.ptr = 0;
            }
            let (ppn, vpn, frame) = &self.inner[self.ptr];
            if Self::has_accessed(memory_set, vpn) {
                Self::clear_accessed(memory_set, vpn);
                self.ptr += 1;
            } else {
                let (ppn, vpn, _) = self.inner.remove(self.ptr).unwrap();
                return (ppn, vpn);
            }
        }
    }

    pub fn work_improve<M: PageManager<Meta>>(&mut self, memory_set: &mut AddressSpace<Meta, M>) -> (PPN<Meta>, VPN<Meta>) {
        if self.ptr >= self.inner.len() {
            self.ptr = 0;
        }
        let l = self.inner.len();
        let add_ptr = |ptr:&usize| if self.ptr == l-1 { 0 } else { self.ptr + 1 };

        // loop 1
        let mut cache = vec![(false, false); l];
        for _ in 0..l {
            let (ppn, vpn, frame) = &self.inner[self.ptr];

            let (accessed, dirty) = Self::get_accessed_dirty(memory_set, vpn);
            if !accessed && !dirty {
                let (ppn, vpn, _) = self.inner.remove(self.ptr).unwrap();
                return (ppn, vpn);
            } else {
                cache[self.ptr] = (accessed, dirty);
            }
            add_ptr(&self.ptr);
        }

        // loop 2
        for _ in 0..l {
            let (accessed, dirty) = cache[self.ptr];
            if dirty && !accessed {
                let (ppn, vpn, _) = self.inner.remove(self.ptr).unwrap();
                return (ppn, vpn);
            } else if accessed {
                let (ppn, vpn, frame) = &self.inner[self.ptr];
                Self::clear_accessed(memory_set, vpn);
            }
            add_ptr(&self.ptr);
        }

        // loop 3
        for _ in 0..l {
            let (_, dirty) = cache[self.ptr];
            if !dirty {
                let (ppn, vpn, _) = self.inner.remove(self.ptr).unwrap();
                return (ppn, vpn);
            }
            add_ptr(&self.ptr);
        }

        // loop 4, actually remove idx=ptr 
        let (ppn, vpn, _) = self.inner.remove(self.ptr).unwrap();
        (ppn, vpn)
    }
}