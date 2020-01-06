#![no_std]
#![no_main]
#![feature(asm)]

mod serial;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_panic_info: &PanicInfo) -> ! {
    loop {}
}

const FIRST_DEVICE_ADDRESS: usize =0xd0000000;
const FIRST_DEVICE_IRQ: u8 = 5;

/*
virtio-mmio: Registering device virtio-mmio.0 at 0xd0000000-0xd0000fff, IRQ 5.
virtio-mmio: Registering device virtio-mmio.1 at 0xd0001000-0xd0001fff, IRQ 6.
*/

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_println!("Hello {} times !", 42);

    let mut result: i32 = 0;
    unsafe {
        //asm!("mov eax, 2" : "={eax}"(result) : : : "intel");
        asm!("int $$0x05" : /* no outputs */ : /* no inputs */ : /*"{eax}"*/);
    }
    serial_println!("eax is currently {}", result);

    /*unsafe {
        //asm!("mov eax, 2" : "={eax}"(result) : : : "intel");
        asm!("int $$0x05" : /* no outputs */ : /* no inputs */ : "intel");
    }*/

    serial_println!("start address : ${:X}", FIRST_DEVICE_ADDRESS);

    serial_println!("I am searching for an mmio device (magic is 0x74726976)...");

    let start_address = FIRST_DEVICE_ADDRESS as *mut u64;
    unsafe  {
        //*start_address = 54;
        //serial_print!("written !!!");
        let value = *start_address;
        serial_print!("{:08x} ", value);
    }

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