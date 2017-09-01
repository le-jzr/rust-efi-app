#![feature(alloc)]
#![feature(allocator_api)]
#![feature(const_fn)]
#![feature(try_from)]

#![no_std]

#![allow(dead_code)]


extern crate efi_types;

pub mod protocol;

use core::mem;

mod allocator;
pub use allocator::{Allocator, FrontAllocator, PhysicalAddress};

mod globals {
    use efi_types;
    use protocol;
    use core::ptr;

    pub(crate) const PAGE_SIZE: usize = 4096;
    pub(crate) static mut SYSTEM_TABLE: *mut efi_types::EFI_SYSTEM_TABLE = ptr::null_mut();
    pub(crate) static mut BOOT_SERVICES_TABLE: Option<&mut protocol::boot_services::BootServices> = None;
    pub(crate) static mut RUNTIME_SERVICES_TABLE: *mut efi_types::EFI_RUNTIME_SERVICES = ptr::null_mut();
}

//#[repr(transparent)]
pub struct Arg1(efi_types::EFI_HANDLE);

//#[repr(transparent)]
pub struct Arg2(*mut efi_types::EFI_SYSTEM_TABLE);

pub struct Status(efi_types::EFI_STATUS);

impl Status {
    pub const fn success() -> Self {
        Status(efi_types::EFI_SUCCESS as efi_types::EFI_STATUS)
    }

    pub fn load_error() -> Self {
        let efi_load_error: efi_types::EFI_STATUS = if mem::size_of::<efi_types::EFI_STATUS>() == 4 {
            0x80000001
        } else {
            0x8000000000000001 as efi_types::EFI_STATUS
        };

        Status(efi_load_error)
    }
}

#[derive(Default)]
struct PBuffer {
    buffer: [u16; 32],
}

pub struct BootContext {
    print_buffer: PBuffer,
}

pub fn __print(s: &str) {
    let out = unsafe{__fixme_temporary_out()};
    out.output_string(s);
}

pub fn __println(s: &str) {
    __print(s);
    __print("\n");
}

pub fn __printx64(num: u64) {
    __print("0x");
    for i in 0..16 {
        __print(match (num >> ((15-i)*4)) & 0xf {
            0 => "0",
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "A",
            11 => "B",
            12 => "C",
            13 => "D",
            14 => "E",
            15 => "F",
            _ => { __println("\n\nUNREACHABLE REACHED\n\n"); unreachable!() },
        });
    }
}

pub fn __printx64ln(num: u64) {
    __printx64(num);
    __print("\n");
}

pub fn __printval(s: &str, num: usize) {
    __print(s);
    __print(": ");
    __printx64ln(num as u64);
}

pub unsafe fn __fixme_temporary_out() -> &'static mut protocol::console::simple_text_output::Protocol {
    &mut *((*globals::SYSTEM_TABLE).ConOut as *mut protocol::console::simple_text_output::Protocol)
}

impl BootContext {
    pub unsafe fn new(_image_handle: Arg1, system_table: Arg2) -> BootContext {
        let Arg2(table) = system_table;
        globals::SYSTEM_TABLE = table;
        globals::BOOT_SERVICES_TABLE = ((*table).BootServices as *mut protocol::boot_services::BootServices).as_mut();
        globals::RUNTIME_SERVICES_TABLE = (*table).RuntimeServices;
        BootContext{ print_buffer: PBuffer::default() }
    }

    pub fn console_out(&mut self) -> &mut protocol::console::simple_text_output::Protocol {
        unsafe { &mut *((*globals::SYSTEM_TABLE).ConOut as *mut protocol::console::simple_text_output::Protocol) }
    }

    pub fn print(&mut self, s: &str) {
        core::mem::drop(self.console_out().output_string(s));
    }

/*

    unsafe fn allocate_pages() {
    }

    unsafe fn free_pages()
    fn get_memory_map() {
    }



    pub fn validate(&self) -> Result<(), ValidateError> {
        unimplemented!()
    }

    pub fn exit(self) -> RuntimeContext {
        unimplemented!()
    }
*/

}

