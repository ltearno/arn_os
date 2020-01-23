#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_x86_interrupt)]

mod bootparam;
mod serial;

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::registers::control::Cr0;
use x86_64::registers::control::Cr3;
use x86_64::registers::control::Cr4;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::structures::paging::PageTable;
use x86_64::structures::paging::PageTableFlags;
use x86_64::{structures::paging::MapperAllSizes, VirtAddr};
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PhysFrame, Size4KiB, UnusedPhysFrame},
    PhysAddr,
};

#[panic_handler]
fn panic(_panic_info: &PanicInfo) -> ! {
    loop {}
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt
    };
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    panic!("Bye\n");
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

pub fn active_level_4_table() -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = phys.as_u64();

    unsafe {
        let page_table_ptr: *mut PageTable = virt as *mut PageTable;

        &mut *page_table_ptr // unsafe
    }
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xd0000000));
    // FIXME: ONLY FOR TEMPORARY TESTING
    let unused_frame = unsafe { UnusedPhysFrame::new(frame) };
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let map_to_result = mapper.map_to(page, unused_frame, flags, frame_allocator);
    map_to_result.expect("map_to failed").flush();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    IDT.load();
    let l4_table = unsafe { active_level_4_table() };

    let (level_4_table_frame, _) = Cr3::read();
    println!(
        "L4 table address: {:X}",
        level_4_table_frame.start_address().as_u64()
    );

    /*let mut frame_allocator = EmptyFrameAllocator;
    let page = Page::containing_address(VirtAddr::new(0xd0000000));
    create_example_mapping(page, &mut offset_page_table, &mut frame_allocator);
    let page_ptr = page.start_address().as_u64();
    println!("page address : ${:X}", page_ptr);*/

    //let a = x86_64::PhysAddr::new(0xd0000000);

    let mut l4idx = 0;
    let mut l3idx = 0;
    let mut l2idx = 0;

    for (i, entry) in l4_table.iter().enumerate() {
        if entry.flags().bits() != 0 {
            println!("L4 Entry {}: {:?}", i, entry);
            unsafe {
                let l3_table = &mut *(entry.addr().as_u64() as *mut PageTable);
                for (i, entry) in l3_table.iter().enumerate() {
                    if entry.flags().bits() != 0 {
                        println!("L3 Entry {}: {:?}", i, entry);
                        if (entry.flags()
                            & x86_64::structures::paging::page_table::PageTableFlags::HUGE_PAGE)
                            != x86_64::structures::paging::page_table::PageTableFlags::HUGE_PAGE
                        {
                            let mut l2_table = &mut *(entry.addr().as_u64() as *mut PageTable);
                            for (i, entry) in l2_table.iter_mut().enumerate() {
                                if entry.flags().bits() != 0 {
                                    if (l4idx == 0 && l3idx == 0 && l2idx == 4) {
                                        entry.set_addr(PhysAddr::new(0xd0000000), entry.flags());
                                    }
                                    println!("L2 Entry {}: {:?}", i, entry);
                                    if (entry.flags() & x86_64::structures::paging::page_table::PageTableFlags::HUGE_PAGE) != x86_64::structures::paging::page_table::PageTableFlags::HUGE_PAGE {
                                            let l1_table =
                                                &mut *(entry.addr().as_u64() as *mut PageTable);
                                            for (i, entry) in l1_table.iter().enumerate() {
                                                if entry.flags().bits() != 0 {
                                                    println!("L1 Entry {}: {:?}", i, entry);
                                                }
                                            }
                                        }
                                }
                                l2idx = l2idx + 1;
                            }
                        }
                    }
                    l3idx = l3idx + 1;
                }
            }
        }
        l4idx = l4idx + 1;
    }

    x86_64::instructions::tlb::flush_all();

    let offset_page_table = unsafe { OffsetPageTable::new(l4_table, VirtAddr::new(0)) };
    let virt = VirtAddr::new(0b100000000000_000000000000); //0b111111111000000000000000000000 + 0x9020);
    let phys = offset_page_table.translate_addr(virt);
    println!("TRANSLATION {:?} -> {:?}", virt, phys);

    let start_address = 0b100000000000_000000000000 as *mut u32;
    unsafe {
        let value = *start_address.offset(0);
        println!("{:X}", value); // should be 'virt'
    }

    let mut result: i32 = 0;
    let cr0 = Cr0::read();
    println!("cr0 is currently {:?}", cr0);
    let (cr3, _) = Cr3::read();
    println!("cr3 is currently {:?}", cr3);
    let cr4 = Cr4::read();
    println!("cr4 is currently {:?}", cr4);
    /*unsafe {
        asm!(r"
        mov %esi, $0
        ": "=r"(result));
        //asm!("cpuid" : "={eax}"(result) : "{eax}"(0x80000000) : : "intel");
        //asm!("int $$0x05" : /* no outputs */ : /* no inputs */ : /*"{eax}"*/);
    }
    println!("eax is currently {:X}", result);*/

    // boot params
    let boot_param_address = ZERO_PAGE_START as *mut bootparam::boot_params;
    unsafe {
        for i in 0..128 {
            let entry = (*boot_param_address).e820_table[i];
            println!(
                "entry {:X} {:X} {:X} {}",
                i, entry.addr, entry.size, entry.type_
            );
            if entry.size == 0 {
                break;
            }
        }
    }

    loop {}
}
