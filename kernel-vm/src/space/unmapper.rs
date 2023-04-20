use crate::{AddressSpace, PageManager};
use core::{ptr::NonNull};
use page_table::{Decorator, Pos, Pte, Update, VmFlags, VmMeta};

pub(super) struct UNMapper<'a, Meta: VmMeta, M: PageManager<Meta>> {
    space: &'a mut AddressSpace<Meta, M>,
    done: bool,
}

impl<'a, Meta: VmMeta, M: PageManager<Meta>> UNMapper<'a, Meta, M> {
    #[inline]
    pub fn new(
        space: &'a mut AddressSpace<Meta, M>
    ) -> Self {
        Self {
            space,
            done: false,
        }
    }

    #[inline]
    pub fn ans(self) -> bool {
        self.done
    }
}

impl<Meta: VmMeta, M: PageManager<Meta>> Decorator<Meta> for UNMapper<'_, Meta, M> {
    #[inline]
    fn arrive(&mut self, pte: &mut Pte<Meta>, _target_hint: Pos<Meta>) -> Pos<Meta> {
        assert!(pte.is_valid());
        let orig_flag = pte.flags();
        let new_flag = orig_flag ^ VmFlags::VALID ;
        *pte = new_flag.build_pte(pte.ppn());
        self.done = true;
        Pos::stop()
    }

    #[inline]
    fn meet(
        &mut self,
        _level: usize,
        pte: Pte<Meta>,
        _target_hint: Pos<Meta>,
    ) -> Option<NonNull<Pte<Meta>>> {
        if self.space.page_manager.check_owned(pte) {
            Some(self.space.page_manager.p_to_v(pte.ppn()))
        } else {
            None
        }
    }

    #[inline]
    fn block(&mut self, _level: usize, _pte: Pte<Meta>, _target_hint: Pos<Meta>) -> Update<Meta> {
        Update::Target(Pos::stop())
    }
}
