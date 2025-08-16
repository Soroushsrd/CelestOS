// since we are writing an os kernel from scratch, we cant use what is available
// in the std library such as prints, vecs and so on!
// also #[test] belongs to std lib so we need to define our own test
// runner. first we set the 2 lines below (regarding tests)
// then we define the test runner at the end
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os::println;
// most languages need a runtime system which is responsible for
// tasks like gc in java or goroutines in go. this runtime will be called
// before main
// rust has a minimal runtime which is responsible for setting up stack overflow guards
// or printing a backgrace on panics. after that it will call the main method.
// rust binaries usually use crt0 which is "C runtime Zero" which puts stack variables in
// their corresponding registers and then calls main.
// Since we dont have access to rust runtime or crt0, we cant have main either!
// so we need to define our own entry point
//
// _start which is entry point will never return because it will not be called by any function
// instead, it will be invoked directly by bootloader or the OS.
// so instead of returning, it will call the exit() syscall
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World!");
    // start the idt
    os::init();
    // invoke a breakpoint exception
    // unsafe {
    //     // triggers a page fault
    //     *(0xdeadbeef as *mut u8) = 42;
    // }

    x86_64::instructions::interrupts::int3();

    // We set the name of the test framework entry function to test_main and call
    // it from our _start entry point. We use conditional compilation to add
    // the call to test_main only in test contexts because the
    // function is not generated on a normal run.
    #[cfg(test)]
    test_main();

    println!("it did not crash!");
    loop {}
}

// panic info contains the file and the line where the panic has occured
// + an optional panic message
// ! means that the method will return the "never type"
// rust uses stack unwinding to run destructors for all stack variables
// in the case of a panic. but unwinding requires os dependant libraries!
// to disable unwinding, we set the [profile.release] or [profile.dev]
// panic attribute to "abort" in cargo.toml
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}
//------------------TESTS----------------------------//

#[test_case]
fn trivial_assertion() {
    let one = 1;
    assert_eq!(one, 1);
}

#[test_case]
fn trivial_assertion2() {
    println!("Test printing something");
}
