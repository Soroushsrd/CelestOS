// cpu exceptions occur in erroneous situations such as accessing an invalid memory address
// or dividing by zero. intterupts are our way or reacting to these errors.
// An interrupt descriptor table will be used for this purpose.
//
// An exception is a signal that something is wrong! and when an exception occurs, the cpu interrupts
// its current work and immediately calls a specific exception handler function based on exception type.
//
// on x86, there are 20 different cpu execiton types, some of which are:
//  1. page faults: this one happens on illegal memmory access.
//  2. invalid Opcode: it occurs when the current instruction is invalid, for example when using an
//      instruction on a machine that doesnt support it!
//  3. general protection fault: this one can happen due to many reasons such as access violations,
//  4. double fault: when an exception occurs, the cpu tries to call the corresponding handler function
//      if another exception occurs while doing that, a double fault exception arrises. also happens
//      when there is no handler function registered for an exception.
//  5. triple fault: if an exception occurs when CPU tries to call the double fault exception handler
//      this one only gets handled by rebooting the system!
//
// ** IDT or Interrupt Descriptor Table **
// This table specifies handler functions for each cpu exception. The hardware will use this table directly
// Each entry must have a 16 byte structure as follows:
// Type	Name	                    Description
// u16	Function Pointer [0:15]	    The lower bits of the pointer to the handler function.
// u16	GDT selector	            Selector of a code segment in the global descriptor table.
// u16	Options	(see below)
// u16	Function Pointer [16:31]	The middle bits of the pointer to the handler function.
// u32	Function Pointer [32:63]	The remaining bits of the pointer to the handler function.
// u32	Reserved
//
// Option field has the following structure:
// Bits	Name	                            Description
// 0-2	Interrupt Stack Table Index	        0: Don’t switch stacks, 1-7: Switch to the n-th stack in the Interrupt Stack Table when this handler is called.
// 3-7	Reserved
// 8	0: Interrupt Gate, 1: Trap Gate	    If this bit is 0, interrupts are disabled when this handler is called.
// 9-11	must be one
// 12	must be zero
// 13‑14  Descriptor Privilege Level (DPL)	The minimal privilege level required for calling this handler.
// 15	Present
//
//
// When an exception occurs, the CPU does the following:
//  1. pushes some registers on the stack, including the instruction pointer and RFLAGS register
//  2. reads the corresponding entry from IDT
//  3. checks if the entry is present and if not, raises a double fault
//  4. disables hardware interrups if the entry is an interrupt gate(0)
//  5. loads the specified global descriptor table (GDT)
//  6. jumps to the specified handler function
//
//
// Exceptions are similar to function calls, CPU jumps to the first instruction of the called function and
// executes it. then jumps to the return address and continues with the parent function
// the difference is that execptions are not voluntarily invoked.
//
// Registers are divided into two parts: preserved and scratch registers.
// The values of preserved registers must remain unchanged across function calls. These are called "callee-saved"
// registers because their callee (called function) can only change their value if they restore their original value before returning.
//
// Scratch registers can be overwritten without restrictions. if the caller(called function) wants to preserve the value of a scratch
// register across a function call, it must backup (into stack) and restore its value before the function call. These are called
// "caller-saved" registers.
//
// preserved registers	                scratch registers
// rbp, rbx, rsp, r12, r13, r14, r15	rax, rcx, rdx, rsi, rdi, r8, r9, r10, r11
// callee-saved	                        caller-saved
//
// Since we dont know when an exception occurs, we cant backup any registers before. this means that we cant use a
// calling convention that relies on caller-saved registers for exception handlers.
// x85-interrupt calling convention guarantees that all register values are restored
// to their original values on function return
//
// This means that compiler backsup registers that are overwritten by the function. This improves efficiency!
//
// ** Interrupt Stack Frame
// on a normal function call (call instruction), the cpu pushes the return address before jumping to the target function
// and when it returns, cpu pops the return address and jumps to it:
//
// stack:
// ------------------- <-------- old stack pointer
// | return address  |         8bytes
// ------------------- <-------- new stack pointer
// | stack frame of  |
// | the handler func|
// ------------------
//
// This simple form wont work for exceptions and interrupts because interrupt handlers often run in another context
// instead, the CPU does the following:
//      1. saves the old stack ptr (rsp and ss registers)
//      2. aligns the stack ptr: some stack pointers must be aligned on a 16 byte boundary
//      3. switches stacks: occurs when cpu privilege level changes.
//      4. pushes the old stack ptr: rsp and ss values are pushed to stack to be restored later
//      5. pushes and updates the RFLAGS register (control and stat bits)
//      6. pushes the instruction ptr: before jumping to interrupt handler func, cpu pushes the rip and cs.
//      7. invokes the interrupt handler: cpu reads the address and segment descriptor of the interrupt handler from
//          corresponding field in the IDT and invokes it by loading the values into rip and cs.
//
// stack:
// --------------------- <-------- old stack ptr
// | stack alignment   |
// ---------------------
// | stack segment (ss)|
// ---------------------
// | stack ptr (RSP)   |
// ---------------------
// | RFLAGS            |  8bytes
// ---------------------
// | Code segment (Cs)|
// ---------------------
// | instruction ptr(rip)|
// ---------------------
// | error code (optional)|
// -------------------- <-------- new stack ptr
// | stack frame of  |
// | the handler func|
// ------------------

use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::println;
// idt must live staticly but should also be mutable. so we use lazy static
// to initialize it at runtime
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    // now we stard adding exception handlers
    // breakpoint exception is the exception used to temporarily pause a program
    // when the breakpoint instruction "int3" is executed
    // loading this idt causes the cpu to use this idt for its instructions
    IDT.load();
}

/// prints exception:breakpoint when a breakpoint exception is invoked!
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
