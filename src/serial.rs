use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // this method will need the address of the first io port
        // of the UART as an argument. it will then calculate the rest of needed
        // ports from this address
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("priting to serial failed");
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => {
        $crate::serial_print!("\n");
    };
    ($fmt:expr) => {
        $crate::serial_print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*)=>{
        $crate::serial_print!(concat!($fmt,"\n"),$($arg)*);
    }
}
