
extern crate alloc;
use self::alloc::allocator::{Alloc, Layout, AllocErr};
use globals;
use protocol::boot_services::{AllocateType, MemoryType, PhysAddr};

pub struct PhysicalAddress(pub u64);

pub unsafe trait FrontAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr>;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
    unsafe fn feed_memory(&mut self, addr: PhysicalAddress, size: usize);
}

pub struct Allocator<F: FrontAllocator> {
    fully_stocked: bool,
    alloc: F,
}

impl<F: FrontAllocator> Allocator<F> {
    pub const fn new(alloc: F) -> Self {
        Allocator{fully_stocked: false, alloc: alloc}
    }

    #[cold]
    #[inline(never)]
    unsafe fn restock(&mut self, size: usize) {
        if self.fully_stocked {
            return;
        }

        if let Some(ref mut table) = globals::BOOT_SERVICES_TABLE {
            // Allocate missing memory from EFI boot services.

            let pages = ((size - 1) / globals::PAGE_SIZE) + 1;
            if let Ok(addr) = table.allocate_pages(AllocateType::AllocateAnyPages, MemoryType::LoaderData, pages, 0) {
                self.alloc.feed_memory(PhysicalAddress(addr as _), pages * globals::PAGE_SIZE);
            }
        } else {
            // We have already exited boot services, but haven't yet scanned
            // memory map for free memory.

            // FIXME: Fill in information from the memory map.
            unimplemented!();
        }
    }
}


unsafe impl<F: FrontAllocator> Alloc for Allocator<F> {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        if let Ok(p) = self.alloc.alloc(layout.clone()) {
            return Ok(p);
        }

        self.restock(layout.size());
        #[cold]
        self.alloc.alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        self.alloc.dealloc(ptr, layout)
    }
}

