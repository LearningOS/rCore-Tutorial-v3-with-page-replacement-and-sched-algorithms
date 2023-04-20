use core::marker::PhantomData;
use alloc::{collections::BTreeMap};
use kernel_vm::{PageManager, AddressSpace, VmMeta, VAddr, VmFlags};
use crate::plugins::Manage;

pub struct PageFaultHandler<Meta: VmMeta, PM: PageManager<Meta>, FM: Manage<Meta, PM>> {
    enable_pagefault: bool,
    // global_manager: Option<FM>,
    managers: BTreeMap<usize, FM>,

    dummy1: PhantomData<Meta>,
    dummy2: PhantomData<PM>
}

impl<Meta: VmMeta, PM: PageManager<Meta>, FM: Manage<Meta, PM> + Clone> PageFaultHandler<Meta, PM, FM> {
    pub const fn new() -> Self {
        if cfg!(feature = "none") {
            Self { enable_pagefault: false, managers: BTreeMap::new(), dummy1: PhantomData::<Meta>, dummy2: PhantomData::<PM> }
        } else {
            Self { enable_pagefault: true, managers: BTreeMap::new(), dummy1: PhantomData::<Meta>, dummy2: PhantomData::<PM> }
        }
    }

    // pub fn set_manager(&mut self, manager: FM) {
    //     self.global_manager = Some(manager);
    // }

    pub fn handle_pagefault(&mut self, addr: usize, flag: usize, memory_set:&mut AddressSpace<Meta, PM>, task_id: usize) {
        if self.enable_pagefault {
            // check if the addr is already mapped to memory set
            let vaddr: VAddr<Meta> = VAddr::from(addr);
            if let Some(pte) = memory_set.translate_to_pte(vaddr) {
                if pte.is_valid() && !pte.flags().contains(unsafe { VmFlags::from_raw(flag) }) {
                    panic!("[PAGE FAULT]: unsupported flags, require={}, orig={}", flag, pte.flags().val());
                }
            }

            // handle page fault
            match self.managers.get_mut(&task_id) {
                None => {
                    panic!("[PAGE FAULT]: frame manager not set");
                },
                Some(fm) => {
                    let vpn = vaddr.floor();
                    fm.handle_pagefault(memory_set, vpn, task_id);
                }
            };
        } else {
            panic!("Page fault but page fault handling is not enabled")
        }
    }

    pub fn new_memory_set(&mut self, task_id: usize) {
        self.managers.insert(task_id, FM::new());
    }

    pub fn clone_memory_set(&mut self, task_id: usize, parent_id: usize) {
        let pmanager = self.managers.get(&parent_id).unwrap();
        self.managers.insert(task_id, pmanager.clone());
    }

    pub fn del_memory_set(&mut self, task_id: usize) {
        self.managers.remove(&task_id);
    }
}