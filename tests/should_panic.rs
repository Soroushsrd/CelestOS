// just to pound this home, each integration test is considered as a seperate process
// thus we need to provide 1.entry point 2.panic handler 3.test runner
// now we could either define any of these again in here or use the ones in lib.rs
#![no_std]
#![no_main]
// no need for the lines below. harness is set to false
// #![feature(custom_test_frameworks)]
// #![test_runner(test_runner)]
// #![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use os::{exit_qemu, serial_print, serial_println};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // test_main();
    // instead of defining a test runner, we disabled harness and used the should fail
    // in here directly. this method can be used for integration tests that have one or two methods max
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(os::QemuExitCode::Failed);
    loop {}
}

// we could either the fine the code below as our test runner or disable the harness attr in cargo.toml
// so that test runners wouldnt be necessary and each test would behave like a normal executable/method
//
// fn test_runner(tests: &[&dyn Fn()]) {
//     serial_println!("Running {} tests", tests.len());

//     for test in tests {
//         test();
//         serial_println!("[Test did not panic!]");
//         exit_qemu(os::QemuExitCode::Failed);
//     }
//     exit_qemu(os::QemuExitCode::Success);
// }

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(os::QemuExitCode::Success);
    loop {}
}

//------------Tests-------------//
// #[test_case]
fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(1, 0);
}
