use efi_types;

use protocol::{Result, status_to_result};

#[repr(C)]
pub enum AllocateType {
    AllocateAnyPages,
    AllocateMaxAddress,
    AllocateAddress,
}

#[repr(C)]
pub enum MemoryType {
    ReservedMemoryType,
    LoaderCode,
    LoaderData,
    BootServicesCode,
    BootServicesData,
    RuntimeServicesCode,
    RuntimeServicesData,
    ConventionalMemory,
    UnusableMemory,
    ACPIReclaimMemory,
    ACPIMemoryNVS,
    MemoryMappedIO,
    MemoryMappedIOPortSpace,
    PalCode,
    PersistentMemory,
    MaxMemoryType,
}

pub type PhysAddr = usize;

#[repr(C)]
pub struct BootServices {
    table: efi_types::EFI_BOOT_SERVICES,
}

impl BootServices {
    pub fn allocate_pages(&mut self, atype: AllocateType, mtype: MemoryType, pages: usize, addr: PhysAddr) -> Result<PhysAddr> {
        let mut addr: efi_types::EFI_PHYSICAL_ADDRESS = addr as _;
        let allocfn = self.table.AllocatePages.unwrap();
        let status = unsafe { allocfn(atype as _, mtype as _, pages as _, &mut addr) };
        status_to_result(status, addr as _)
    }
}
