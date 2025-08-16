# Rust Based OS Kernel

A bare-metal operating system kernel written in Rust, following the excellent [Writing an OS in Rust](https://os.phil-opp.com/) blog series by Philipp Oppermann.

##  Current Features

- **VGA Text Mode**: Custom VGA buffer implementation for kernel output
- **Exception Handling**: Complete Interrupt Descriptor Table (IDT) setup
- **Double Fault Prevention**: Task State Segment (TSS) with Interrupt Stack Table (IST)
- **Serial Communication**: UART support for debugging and testing
- **Custom Test Framework**: Integration and unit testing without std library
- **Memory Safety**: Volatile memory access and proper stack overflow protection

##  Building and Running

### Prerequisites

- Rust nightly toolchain
- `bootimage` crate for creating bootable disk images
- QEMU for emulation

### Setup

```bash
# Install required components
rustup component add rust-src
cargo install bootimage

# Build the kernel
cargo build

# Run in QEMU
cargo run

# Run tests
cargo test
```

### Configuration

The project uses custom target specification (`x86_64-os.json`) and requires:
- `#![no_std]` - No standard library
- `#![no_main]` - Custom entry point
- Nightly Rust features for interrupt handling


## License

This project is created for educational purposes following the MIT-licensed blog series.


*Work in Progress*
