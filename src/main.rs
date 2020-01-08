#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_x86_interrupt)]

mod bootparam;
mod serial;

use core::panic::PanicInfo;
use x86_64::registers::control::Cr0;
use x86_64::registers::control::Cr3;
use x86_64::registers::control::Cr4;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::structures::paging::PageTable;
use x86_64::structures::paging::PageTableFlags as Flags;
use x86_64::{structures::paging::MapperAllSizes, VirtAddr};
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PhysFrame, Size4KiB, UnusedPhysFrame},
    PhysAddr,
};

#[panic_handler]
fn panic(_panic_info: &PanicInfo) -> ! {
    loop {}
}

use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.double_fault.set_handler_fn(double_fault_handler); // new
        idt
    };
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    serial_println!("EXCEPTION: PAGE FAULT");
    serial_println!("Accessed Address: {:?}", Cr2::read());
    serial_println!("Error Code: {:?}", error_code);
    serial_println!("{:#?}", stack_frame);
    panic!("EXCEPTION: PAGE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// physical addresses
const FIRST_DEVICE_ADDRESS: usize = 0xd0000000;
const FIRST_DEVICE_IRQ: u8 = 5;

// boot params
const ZERO_PAGE_START: usize = 0x7000;

/*
virtio-mmio: Registering device virtio-mmio.0 at 0xd0000000-0xd0000fff, IRQ 5.
virtio-mmio: Registering device virtio-mmio.1 at 0xd0001000-0xd0001fff, IRQ 6.
*/

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame> {
        None
    }
}

pub unsafe fn active_level_4_table() -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = phys.as_u64();
    let page_table_ptr: *mut PageTable = virt as *mut PageTable;

    &mut *page_table_ptr // unsafe
}

pub unsafe fn get_offset_page_table() -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table();
    OffsetPageTable::new(level_4_table, VirtAddr::new(0))
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xd0000000));
    // FIXME: ONLY FOR TEMPORARY TESTING
    let unused_frame = unsafe { UnusedPhysFrame::new(frame) };
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = mapper.map_to(page, unused_frame, flags, frame_allocator);
    map_to_result.expect("map_to failed").flush();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    IDT.load();

    let l4_table = unsafe { active_level_4_table() };
    let mut offset_page_table = unsafe { OffsetPageTable::new(l4_table, VirtAddr::new(0)) };

    //let mut offset_page_table = unsafe { get_offset_page_table() };
    let virt = VirtAddr::new(0xa000); //0b111111111000000000000000000000 + 0x9020);
    let phys = offset_page_table.translate_addr(virt);
    serial_println!("TRANSLATION {:?} -> {:?}", virt, phys);
    //serial_println!("TABLE {:?}", offset_page_table);

    serial_println!("Hello {} times !", 42);

    //let a = x86_64::PhysAddr::new(0xd0000000);

    for (i, entry) in l4_table.iter().enumerate() {
        if entry.flags().bits() != 0 {
            serial_println!("L4 Entry {}: {:?}", i, entry);
            unsafe {
                let l3_table = &mut *(entry.addr().as_u64() as *mut PageTable);
                for (i, entry) in l3_table.iter().enumerate() {
                    if entry.flags().bits() != 0 {
                        serial_println!("L3 Entry {}: {:?}", i, entry);
                        if (entry.flags()
                            & x86_64::structures::paging::page_table::PageTableFlags::HUGE_PAGE)
                            != x86_64::structures::paging::page_table::PageTableFlags::HUGE_PAGE
                        {
                            unsafe {
                                let l2_table = &mut *(entry.addr().as_u64() as *mut PageTable);
                                for (i, entry) in l2_table.iter().enumerate() {
                                    if entry.flags().bits() != 0 {
                                        serial_println!("L2 Entry {}: {:?}", i, entry);
                                        if (entry.flags() & x86_64::structures::paging::page_table::PageTableFlags::HUGE_PAGE) != x86_64::structures::paging::page_table::PageTableFlags::HUGE_PAGE
                                        {
                                        unsafe {
                                            let l1_table =
                                                &mut *(entry.addr().as_u64() as *mut PageTable);
                                            for (i, entry) in l1_table.iter().enumerate() {
                                                if entry.flags().bits() != 0 {
                                                    serial_println!("L1 Entry {}: {:?}", i, entry);
                                                }
                                            }
                                        }
                                    }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut result: i32 = 0;
    let (cr3, _) = Cr3::read();
    serial_println!("cr3 is currently {:?}", cr3);
    let cr0 = Cr0::read();
    serial_println!("cr0 is currently {:?}", cr0);
    let cr4 = Cr4::read();
    serial_println!("cr4 is currently {:?}", cr4);
    unsafe {
        asm!(r"
        mov %esi, $0
        ": "=r"(result));
        //asm!("cpuid" : "={eax}"(result) : "{eax}"(0x80000000) : : "intel");
        //asm!("int $$0x05" : /* no outputs */ : /* no inputs */ : /*"{eax}"*/);
    }
    serial_println!("eax is currently {:X}", result);

    let boot_param_address = ZERO_PAGE_START as *mut bootparam::boot_params;
    unsafe {
        for i in 0..128 {
            let entry = (*boot_param_address).e820_table[i];
            serial_println!(
                "entry {:X} {:X} {:X} {}",
                i,
                entry.addr,
                entry.size,
                entry.type_
            );
            if entry.size == 0 {
                break;
            }
        }
    }
    let start_address = 0x20000 as *mut u8;
    let mut i = 0;
    loop {
        unsafe {
            let value = *start_address.offset(i);
            //serial_print!("{:X}", value);
            serial::print_raw(value);
            if (value == 0) {
                break;
            }
        }
        i = i + 1;
    }

    serial_println!("");

    /*let mut frame_allocator = EmptyFrameAllocator;
    let page = Page::containing_address(VirtAddr::new(0xd0000000));
    create_example_mapping(page, &mut offset_page_table, &mut frame_allocator);
    let page_ptr = page.start_address().as_u64();
    serial_println!("page address : ${:X}", page_ptr);*/

    
    serial_println!("start address : ${:X}", FIRST_DEVICE_ADDRESS);
    serial_println!("I am searching for an mmio device (magic is 0x74726976)...");
    let start_address = FIRST_DEVICE_ADDRESS as *mut u8;
    unsafe {
        // *start_address = 54;
        //serial_print!("written !!!");
        let value = *start_address.offset(0);
        serial_print!("{:08x} ", value);
    }

    /*unsafe {
        //asm!("mov eax, 2" : "={eax}"(result) : : : "intel");
        asm!("int $$0x05" : /* no outputs */ : /* no inputs */ : "intel");
    }*/

    /*let mut i = 0;
    loop {
        for ofs in 0..7 {
            unsafe  {
                let value = *start_address.offset(i+ofs);
                serial_print!("{:X} ", value)
                //if (value == 0x74 && *start_address.offset(i+1)==0x72)
                //    || (value == 0x72 && *start_address.offset(i+1)==0x74) {
                //    serial_println!("{:X}: {:X} {:X} {:X} {:X}", i, value, *start_address.offset(i+1), *start_address.offset(i+2), *start_address.offset(i+3));
                //}
                //if *start_address.offset(i)==0x74726976 {
                //    serial_println!("found at 0x{:X}", i);
                //}
            }
        }

        i = i+8;

        serial_println!("");
    }*/

    loop {}
}
