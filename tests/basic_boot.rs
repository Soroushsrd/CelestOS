// The convention for integration tests in Rust is to put them into a tests directory in the project root
// (i.e., next to the src directory). Both the default test framework and custom test
// frameworks will automatically pick up and execute all tests in that directory.
// All integration tests are their own executables and completely separate from our main.rs.
// This means that each test needs to define its own entry point function
#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]
// Each integration test is a separate binary that needs:

// Its own entry point (_start)
// Its own panic handler
// Access to your kernel's functionality

// The Solution
// By putting the test infrastructure in lib.rs, you can reuse it across multiple integration tests:

use core::panic::PanicInfo;

use os::{Testable, println};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

//------------Tests-------------//
#[test_case]
fn test_print() {
    println!("Print testing");
}
