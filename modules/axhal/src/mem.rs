//! Physical memory management.

pub use axplat::mem::{
    MemRegionFlags, PhysMemRegion, mmio_ranges, phys_ram_ranges, phys_to_virt,
    reserved_phys_ram_ranges, total_ram_size, virt_to_phys,
};
use axplat::mem::{check_sorted_ranges_overlap, ranges_difference};
use heapless::Vec;
use lazyinit::LazyInit;
pub use memory_addr::{PAGE_SIZE_4K, PhysAddr, PhysAddrRange, VirtAddr, VirtAddrRange, pa, va};

const MAX_REGIONS: usize = 128;

static ALL_MEM_REGIONS: LazyInit<Vec<PhysMemRegion, MAX_REGIONS>> = LazyInit::new();

#[inline(always)]
fn sym_addr(sym: unsafe extern "C" fn()) -> usize {
    sym as *const () as usize
}

/// Returns an iterator over all physical memory regions.
pub fn memory_regions() -> impl Iterator<Item = PhysMemRegion> {
    ALL_MEM_REGIONS.iter().cloned()
}

/// Fills the `.bss` section with zeros.
///
/// It requires the symbols `_sbss` and `_ebss` to be defined in the linker
/// script.
///
/// # Safety
///
/// This function is unsafe because it writes `.bss` section directly.
pub unsafe fn clear_bss() {
    unsafe {
        let sbss = sym_addr(_sbss);
        let ebss = sym_addr(_ebss);
        core::slice::from_raw_parts_mut(sbss as *mut u8, ebss - sbss).fill(0);
    }
}

/// Initializes physical memory regions.
pub fn init() {
    let mut all_regions = Vec::new();
    let mut push = |r: PhysMemRegion| {
        if r.size > 0 {
            all_regions.push(r).expect("too many memory regions");
        }
    };

    // Push regions in kernel image
    push(PhysMemRegion {
        paddr: virt_to_phys((sym_addr(_stext)).into()),
        size: sym_addr(_etext) - sym_addr(_stext),
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::EXECUTE,
        name: ".text",
    });
    push(PhysMemRegion {
        paddr: virt_to_phys((sym_addr(_srodata)).into()),
        size: sym_addr(_erodata) - sym_addr(_srodata),
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ,
        name: ".rodata",
    });
    push(PhysMemRegion {
        paddr: virt_to_phys((sym_addr(_sdata)).into()),
        size: sym_addr(_edata) - sym_addr(_sdata),
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: ".data .tdata .tbss .percpu",
    });
    push(PhysMemRegion {
        paddr: virt_to_phys((sym_addr(boot_stack)).into()),
        size: sym_addr(boot_stack_top) - sym_addr(boot_stack),
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: "boot stack",
    });
    push(PhysMemRegion {
        paddr: virt_to_phys((sym_addr(_sbss)).into()),
        size: sym_addr(_ebss) - sym_addr(_sbss),
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: ".bss",
    });

    // Push MMIO & reserved regions
    for &(start, size) in mmio_ranges() {
        push(PhysMemRegion::new_mmio(start, size, "mmio"));
    }
    for &(start, size) in reserved_phys_ram_ranges() {
        push(PhysMemRegion::new_reserved(start, size, "reserved"));
    }

    // Combine kernel image range and reserved ranges
    let kernel_start = virt_to_phys(va!(sym_addr(_skernel))).as_usize();
    let kernel_size = sym_addr(_ekernel) - sym_addr(_skernel);
    let mut reserved_ranges = reserved_phys_ram_ranges()
        .iter()
        .cloned()
        .chain(core::iter::once((kernel_start, kernel_size))) // kernel image range is also reserved
        .collect::<Vec<_, MAX_REGIONS>>();

    // Remove all reserved ranges from RAM ranges, and push the remaining as free
    // memory
    reserved_ranges.sort_unstable_by_key(|&(start, _size)| start);
    ranges_difference(phys_ram_ranges(), &reserved_ranges, |(start, size)| {
        push(PhysMemRegion::new_ram(start, size, "free memory"));
    })
    .inspect_err(|(a, b)| error!("Reserved memory region {:#x?} overlaps with {:#x?}", a, b))
    .unwrap();

    // Check overlapping
    all_regions.sort_unstable_by_key(|r| r.paddr);
    check_sorted_ranges_overlap(all_regions.iter().map(|r| (r.paddr.into(), r.size)))
        .inspect_err(|(a, b)| error!("Physical memory region {:#x?} overlaps with {:#x?}", a, b))
        .unwrap();

    ALL_MEM_REGIONS.init_once(all_regions);
}

unsafe extern "C" {
    fn _stext();
    fn _etext();
    fn _srodata();
    fn _erodata();
    fn _sdata();
    fn _edata();
    fn _sbss();
    fn _ebss();
    fn _skernel();
    fn _ekernel();
    fn boot_stack();
    fn boot_stack_top();
}
