// to print a char to the screen in VGA text mode, we have to write it to the text buffer of the
// VGA hardware.
// this buffer is a two dimensional array with 25 rows and 80 columns which directly renders to screen
// each array entry describes a single screen char:
//
// Bit(s)	Value
// 0-7	    ASCII code point
// 8-11	    Foreground color
// 12-14	Background color
// 15	    Blink
//
// so in this way, the first byte represents the char that must be printed
//  the second byte represents how the char is displayed (first 4 bits is fg color and next 3bits bg)
// last bit will determine if the char should blink
// Number	Color	    Number + Bright Bit	    Bright Color
// 0x0	    Black	    0x8	                    Dark Gray
// 0x1	    Blue	    0x9	                    Light Blue
// 0x2	    Green	    0xa	                    Light Green
// 0x3	    Cyan	    0xb	                    Light Cyan
// 0x4	    Red	        0xc	                    Light Red
// 0x5	    Magenta	    0xd	                    Pink
// 0x6	    Brown	    0xe	                    Yellow
// 0x7	    LightGray	0xf	                    White
//
// Bit 4 is the bright bit, which turns, for example,
//  blue into light blue. For the background color, this bit is repurposed as the blink bit.
// The VGA text buffer is accessible via memory-mapped I/O to the address 0xb8000.
// This means that reads and writes to that address don’t access the RAM but directly
// access the text buffer on the VGA hardware.

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_pos: 0,
        color_code: ColorCode::new(Color::Cyan, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// The problem is that we only write to the Buffer and never read from it again.
// The compiler doesn’t know that we really access VGA buffer memory (instead of normal RAM)
// and knows nothing about the side effect that some characters appear on the screen.
// So it might decide that these writes are unnecessary and can be omitted. To avoid this erroneous
// optimization, we need to specify these writes as volatile. This tells the compiler that
// the write has side effects and should not be optimized away.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// always writes to the last line and shifts lines up when a line
/// is full or on \n
pub struct Writer {
    ///keeps track of current position in the last row
    column_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_pos >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_pos;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_char: byte,
                    color_code,
                });
                self.column_pos += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                //ascii chars can already be printed
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not printable ascii range
                _ => self.write_byte(0xfe),
            }
        }
    }
    /// We iterate over all the screen characters and move each character one row up.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(char);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_pos = 0;
    }
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_char: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
    // pub fn print_something() {
    //     use core::fmt::Write;
    //     let mut writer = Writer {
    //         column_pos: 0,
    //         color_code: ColorCode::new(Color::Yellow, Color::Black),
    //         buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    //     };

    //     writer.write_byte(b'H');
    //     writer.write_string("ello! ");
    //     write!(writer, "The numbers are {} and {}", 42, 1.0 / 3.0).unwrap();
    // }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        ($crate::vga_buffer::_print(format_args!($($arg)*)))
    };
}
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*)=>{
        ($crate::print!("{}\n",format_args!($($arg)*)))
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
