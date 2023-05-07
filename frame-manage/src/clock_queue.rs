use alloc::collections::VecDeque;
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

    fn clear_accessed_and_dirty<M: PageManager<Meta>>(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) {
        memory_set.clear_accessed_and_dirty(*vpn);
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
        loop {
            if self.ptr >= self.inner.len() {
                self.ptr = 0;
            }
            let (ppn, vpn, frame) = &self.inner[self.ptr];

            let (accessed, dirty) = Self::get_accessed_dirty(memory_set, vpn);
            if accessed || dirty {
                Self::clear_accessed_and_dirty(memory_set, vpn);
                self.ptr += 1;
            } else {
                let (ppn, vpn, _) = self.inner.remove(self.ptr).unwrap();
                return (ppn, vpn);
            }
        }
    }
}