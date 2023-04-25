use crate::{AddressSpace, PageManager, };
use core::{ptr::NonNull};
use page_table::{Decorator, Pos, Pte, Update, VmFlags, VmMeta};

const ACCESS_FLAG: usize = 1 << 6;
const DIRTY_FLAG: usize = 1 << 7;

pub(super) struct Cleaner<'a, Meta: VmMeta, M: PageManager<Meta>> {
    space: &'a mut AddressSpace<Meta, M>,
    clear_access: bool,
    clear_dirty: bool,
    done: bool,
}

impl<'a, Meta: VmMeta, M: PageManager<Meta>> Cleaner<'a, Meta, M> {
    #[inline]
    pub fn new(
        space: &'a mut AddressSpace<Meta, M>,
        clear_access: bool,
        clear_dirty: bool
    ) -> Self {
        Self {
            space: space,
            clear_access: clear_access,
            clear_dirty: clear_dirty,
            done: false,
        }
    }

    #[inline]
    pub fn ans(self) -> bool {
        self.done
    }
}

impl<Meta: VmMeta, M: PageManager<Meta>> Decorator<Meta> for Cleaner<'_, Meta, M> {
    #[inline]
    fn arrive(&mut self, pte: &mut Pte<Meta>, _target_hint: Pos<Meta>) -> Pos<Meta> {
        assert!(pte.is_valid());
        let mut flag = pte.flags();
        let acc_flag = unsafe { VmFlags::from_raw(ACCESS_FLAG) };
        let dirty_flag = unsafe { VmFlags::from_raw(DIRTY_FLAG) };
        
        if self.clear_access && (flag & acc_flag).val() != 0  {
            flag = flag ^ acc_flag;
        }

        if self.clear_dirty && (flag & dirty_flag).val() != 0 {
            flag = flag ^ dirty_flag;
        }
        
        *pte = flag.build_pte(pte.ppn());
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
