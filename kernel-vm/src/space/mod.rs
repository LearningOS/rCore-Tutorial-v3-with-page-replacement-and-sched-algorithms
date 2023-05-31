mod mapper;
mod visitor;
mod unmapper;
mod flag_cleaner;

extern crate alloc;

use crate::{PageManager, space::flag_cleaner::Cleaner};
use alloc::vec::Vec;
use core::{fmt, ops::Range, ptr::NonNull};
use mapper::Mapper;
use unmapper::UNMapper;
use page_table::{PageTable, PageTableFormatter, Pos, VAddr, VmFlags, VmMeta, PPN, VPN, Pte};
use visitor::Visitor;

pub struct AreaBlock<Meta: VmMeta> {
    pub range: Range<VPN<Meta>>,
    pub flags: VmFlags<Meta>
}

/// 地址空间。
pub struct AddressSpace<Meta: VmMeta, M: PageManager<Meta>> {
    /// 虚拟地址块
    pub areas: Vec<AreaBlock<Meta>>,
    page_manager: M,
}

impl<Meta: VmMeta, M: PageManager<Meta>> AddressSpace<Meta, M> {
    /// 创建新地址空间。
    #[inline]
    pub fn new() -> Self {
        Self {
            areas: Vec::new(),
            page_manager: M::new_root(),
        }
    }

    /// 地址空间根页表的物理页号。
    #[inline]
    pub fn root_ppn(&self) -> PPN<Meta> {
        self.page_manager.root_ppn()
    }

    /// 地址空间根页表
    #[inline]
    pub fn root(&self) -> PageTable<Meta> {
        unsafe { PageTable::from_root(self.page_manager.root_ptr()) }
    }

    /// 向地址空间增加映射关系。
    pub fn map_extern(&mut self, range: Range<VPN<Meta>>, pbase: PPN<Meta>, flags: VmFlags<Meta>) {
        self.areas.push(AreaBlock { range: range.start..range.end, flags: flags });
        let count = range.end.val() - range.start.val();
        let mut root = self.root();
        let mut mapper = Mapper::new(self, pbase..pbase + count, flags);
        root.walk_mut(Pos::new(range.start, 0), &mut mapper);
        if !mapper.ans() {
            // 映射失败，需要回滚吗？
            todo!()
        }
    }

    /// 向已有的 range 增加映射关系
    pub fn map_to_exist_range(&mut self, range_id: usize, vpn: VPN<Meta>, ppn: PPN<Meta>) {
        let mut root = self.root();
        let mut mapper = Mapper::new(self, ppn..ppn+1, self.areas[range_id].flags);
        root.walk_mut(Pos::new(vpn, 0), &mut mapper);

        if !mapper.ans() {
            panic!("map to exist range fail");
        }
    }

    /// 从已有的 range 中删除某一个页面的映射
    pub fn unmap_one_in_exist_range(&mut self, vpn: VPN<Meta>) {
        let mut root = self.root();
        let mut unmapper = UNMapper::new(self);
        root.walk_mut(Pos::new(vpn, 0), &mut unmapper);
        if !unmapper.ans() {
            // unmap fail
            todo!()
        }
    }

    /// 改变某 vpn 对应的 pte 的 flag 的 access 位
    pub fn clear_accessed(&mut self, vpn: VPN<Meta>) {
        let mut root = self.root();
        let mut cleaner = Cleaner::new(self, true, false);
        root.walk_mut(Pos::new(vpn, 0), &mut cleaner);
        if !cleaner.ans() {
            // unmap fail
            todo!()
        }
    }

    /// 分配新的物理页，拷贝数据并建立映射。
    pub fn map(
        &mut self,
        range: Range<VPN<Meta>>,
        data: &[u8],
        offset: usize,
        mut flags: VmFlags<Meta>,
    ) {
        let count = range.end.val() - range.start.val();
        let size = count << Meta::PAGE_BITS;
        assert!(size >= data.len() + offset);
        let page = self.page_manager.allocate(count, &mut flags);
        unsafe {
            use core::slice::from_raw_parts_mut as slice;
            let mut ptr = page.as_ptr();
            slice(ptr, offset).fill(0);
            ptr = ptr.add(offset);
            slice(ptr, data.len()).copy_from_slice(data);
            ptr = ptr.add(data.len());
            slice(ptr, page.as_ptr().add(size).offset_from(ptr) as _).fill(0);
        }
        self.map_extern(range, self.page_manager.v_to_p(page), flags)
    }

    /// map without data, 不 alloc frames 和修改页表，只声明这些地址被进程占有
    /// 之后访问这些页面时必然触发 page fault
    pub fn map_without_data_and_alloc(&mut self, range: Range<VPN<Meta>>, flags: VmFlags<Meta>) {
        self.areas.push(AreaBlock { range: range, flags: flags });
    }

    /// 检查 `flags` 的属性要求，然后将地址空间中的一个虚地址翻译成当前地址空间中的指针。
    pub fn translate<T>(&self, addr: VAddr<Meta>, flags: VmFlags<Meta>) -> Option<NonNull<T>> {
        let mut visitor = Visitor::new(self);
        self.root().walk(Pos::new(addr.floor(), 0), &mut visitor);
        visitor
            .ans()
            .filter(|pte| pte.flags().contains(flags))
            .map(|pte| unsafe {
                NonNull::new_unchecked(
                    self.page_manager
                        .p_to_v::<u8>(pte.ppn())
                        .as_ptr()
                        .add(addr.offset())
                        .cast(),
                )
            })
    }

    /// 返回 pte，并且不检查 `flags` 的 translate
    pub fn translate_to_pte(&self, addr: VAddr<Meta>) -> Option<Pte<Meta>> {
        let mut visitor = Visitor::new(self);
        self.root().walk(Pos::new(addr.floor(), 0), &mut visitor);
        visitor.ans()
    }

    /// 遍历地址空间，将其中的地址映射添加进自己的地址空间中，重新分配物理页并拷贝所有数据及代码
    pub fn cloneself(&self, new_addrspace: &mut AddressSpace<Meta, M>) {
        let root = self.root();
        let areas = &self.areas;
        for (_, area) in areas.iter().enumerate() {
            let range = &area.range;
            let mut visitor = Visitor::new(self);
            // 虚拟地址块的首地址的 vpn
            let vpn = range.start;
            // 利用 visitor 访问页表，并获取这个虚拟地址块的页属性
            root.walk(Pos::new(vpn, 0), &mut visitor);
            // 利用 visitor 获取这个虚拟地址块的页属性，以及起始地址
            let (mut flags, mut data_ptr) = visitor
                .ans()
                .filter(|pte| pte.is_valid())
                .map(|pte| {
                    (pte.flags(), unsafe {
                        NonNull::new_unchecked(self.page_manager.p_to_v::<u8>(pte.ppn()).as_ptr())
                    })
                })
                .unwrap();
            let vpn_range = range.start..range.end;
            // 虚拟地址块中页数量
            let count = range.end.val() - range.start.val();
            let size = count << Meta::PAGE_BITS;
            // 分配 count 个 flags 属性的物理页面
            let paddr = new_addrspace.page_manager.allocate(count, &mut flags);
            let ppn = new_addrspace.page_manager.v_to_p(paddr);
            unsafe {
                use core::slice::from_raw_parts_mut as slice;
                let data = slice(data_ptr.as_mut(), size);
                let ptr = paddr.as_ptr();
                slice(ptr, size).copy_from_slice(data);
            }
            new_addrspace.map_extern(vpn_range, ppn, flags);
        }
    }
}

impl<Meta: VmMeta, P: PageManager<Meta>> fmt::Debug for AddressSpace<Meta, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "root: {:#x}", self.root_ppn().val())?;
        write!(
            f,
            "{:?}",
            PageTableFormatter {
                pt: self.root(),
                f: |ppn| self.page_manager.p_to_v(ppn)
            }
        )
    }
}
