#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga_buffer;

use core::panic::PanicInfo;
use x86_64::instructions::port::Port;

/// uses the port mapped io bus to communicate with Qemu
/// when (value << 1) | 1 is written in Qemu io port, it will
/// exit with a (1<<1)|1=3 status number
/// represents u32 because iosize is 4 bytes
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        // 0xf4 is set in cargo.toml as the io mapped port for qemu
        // as iobase
        let mut port = Port::new(0xf4);
        // we use u32 because we set iosize as 4 bytes (0x04)
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[Ok]");
    }
}

// The custom test frameworks feature generates a main function that calls test_runner,
// but this function is ignored because we use the #[no_main]
// attribute and provide our own entry poin
pub fn test_runner(tests: &[&dyn Testable]) {
    // instead of println, we use serial_print so that it would print
    // to our system stdout instead of the kernel itself
    // println!("Running {} tests", tests.len());
    // remember to ser -serial and -stdin flags in cargo.toml for test-args
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
}

// entry point for cargo test
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
