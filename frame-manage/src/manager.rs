use core::marker::PhantomData;
use kernel_vm::{PageManager, AddressSpace, VmMeta, VAddr, VmFlags};
use crate::plugins::Manage;
use crate::config::GLOBAL_ID;
use crate::virt_frame_swapper::IDE_MANAGER;

pub struct PageFaultHandler<Meta: VmMeta, PM: PageManager<Meta> + 'static, FM: Manage<Meta, PM>, Func: Fn(usize) -> &'static mut AddressSpace<Meta, PM>> {
    enable_pagefault: bool,
    manager: Option<FM>,
    func_get_memory_set: Option<Func>,

    dummy1: PhantomData<Meta>,
    dummy2: PhantomData<PM>
}

impl<Meta: VmMeta, PM: PageManager<Meta> + 'static, FM: Manage<Meta, PM>, Func: Fn(usize) -> &'static mut AddressSpace<Meta, PM>> 
    PageFaultHandler<Meta, PM, FM, Func> {
    pub const fn new() -> Self {
        Self { 
            enable_pagefault: if cfg!(feature = "none") { false } else { true }, 
            manager: None, 
            func_get_memory_set: None, 
            dummy1: PhantomData::<Meta>, 
            dummy2: PhantomData::<PM> 
        }
    }

    pub fn init(&mut self) {
        self.manager = Some(FM::new());
    }

    pub fn set_func(&mut self, func: Func) {
        self.func_get_memory_set = Some(func);
    }

    pub fn handle_pagefault(&mut self, addr: usize, flag: usize, task_id: usize) {
        let get_memory_set = self.func_get_memory_set.as_ref().unwrap();
        if self.enable_pagefault {
            // check if the addr is already mapped to memory set
            let vaddr: VAddr<Meta> = VAddr::from(addr);
            if let Some(pte) = get_memory_set(task_id).translate_to_pte(vaddr) {
                if pte.is_valid() && !pte.flags().contains(unsafe { VmFlags::from_raw(flag) }) {
                    panic!("[PAGE FAULT]: unsupported flags, require={}, orig={}", flag, pte.flags().val());
                }
            }

            // handle page fault
            let vpn = vaddr.floor();
            self.manager.as_mut().expect("manager not init").handle_pagefault(get_memory_set, vpn, task_id);
        } else {
            panic!("Page fault but page fault handling is not enabled")
        }
    }

    pub fn del_memory_set(&mut self, task_id: usize) {
        self.manager.as_mut().expect("manager not init").clear_frames(task_id);
        unsafe { IDE_MANAGER.clear_disk_frames(task_id); }
    }

    pub fn time_interrupt_hook(&mut self) {
        self.manager.as_mut().expect("manager not init").handle_time_interrupt(self.func_get_memory_set.as_ref().unwrap());
    }
}