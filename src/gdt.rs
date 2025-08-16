//
// **Kernel Stack Overflow
//
// A guard page is a memory page at the bottom of a stack that makes it possible to detect stack overflows
// this page is not mapped to any physical frame, thus accessing it causes a page fault.
// Bootloader sets this guard page up for our kernel so a stack overflow causes a page fault
//
// What Happens During Stack Overflow?
// Here's the cascade of events when the kernel's stack overflows:
//
// Step 1: Initial Stack Overflow
// The kernel runs out of stack space and tries to use the guard page
// Since the guard page isn't real memory, this triggers a page fault
//
// Step 2: The First Attempt to Handle the Error
// To handle any error, the CPU needs to save information about what went wrong
// Problem: It tries to save this information on the same broken stack!
// This causes a second page fault
//
//Step 3: Double Fault
// Two page faults in a row create a double fault
// Same problem: It still needs to use the broken stack to save information
// This causes a third page fault
//
// Step 4: Triple Fault and System Reboot
// Three faults in a row create a triple fault
// At this point, the CPU gives up completely
// The system automatically reboots because it can't recover
//
// to fix this issue, we need to make sure that the satck is always valid when a double fault exception
// occurs.
//
// The x86_64 architecture is able to switch to a predefined, known-good stack when an exception occurs
// This switch happens at hardlevel layer so it can hppen before the cpu pushes the exception stack frame
//
// This switching mechanism is implemented as an Interrupt Stack Table (IST). its a table of 7 ptrs to
// known-good stacks
// So for each exception handler, we can choose a stack from IST stack pointers field in the corresponding
// IDT entry. then cpu swtiches to this stack when ever a double fault (ex) occurs. this switch could
// happen before anything is pushed, preventing triple fault.
//
// IST is part of Task State Segment (TSS).
// TSS used to hold various info but after 64-bit archs, its format has changed.
// On x86_64, the TSS holds only ywo stack tables and some other info:
//
// The 64-bit TSS has the following format:
// Field	                Type
// (reserved)	            u32
// Privilege Stack Table	[u64; 3]
// (reserved)	            u64
// Interrupt Stack Table   	[u64; 7]
// (reserved)	            u64
// (reserved)	            u16
// I/O Map Base Address 	u16
//
// privilege stack table is used by CPU when the privilege level is changed.

// GDT or Global Descriptor Table
// it was used for memory segmentation before paging became a thing, but its still used in 64 bit mode
// for various stuff like kernel/user mode config/switching or TSS loading

use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::instructions::{segmentation::Segment, tables::load_tss};
use x86_64::registers::segmentation::CS;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        // defining the 0th IST entry as double fault stack
        // then assigning the top addr of this stack to IST[0]
        // the reasoning behind assigning the top address is that
        // stack grows downwards!
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            //stack end
            stack_start + STACK_SIZE as u64

        };
        tss
    };

    static ref GDT: (GlobalDescriptorTable,Selectors) = {
            let mut gdt = GlobalDescriptorTable::new();

            // CODE SELECTOR EXPLANATION:
            // In x86_64, even though we primarily use paging for memory management,
            // we still need at least one code segment descriptor in the GDT.
            // This is because:
            // 1. The CPU still checks segment registers during certain operations
            // 2. The CS (Code Segment) register must point to a valid code descriptor
            // 3. This descriptor defines privilege levels (ring 0 for kernel, ring 3 for user)
            // 4. When switching between kernel and user mode, the CPU uses these descriptors
            // 5. Some CPU instructions and interrupt handling rely on segment information
            // Without a proper code segment, the CPU would fault when trying to execute code
            let code_selector=gdt.append(Descriptor::kernel_code_segment());

            // TSS SELECTOR EXPLANATION:
            // The TSS (Task State Segment) selector is crucial because:
            // 1. The TSS contains our Interrupt Stack Table (IST) that we just set up
            // 2. The CPU needs to know WHERE to find the TSS in memory
            // 3. A GDT entry acts like a "pointer" that tells the CPU the TSS location and size
            // 4. When a double fault occurs, the CPU looks up the IST through this TSS descriptor
            // 5. Without loading the TSS selector, the CPU wouldn't know about our safe stack
            // 6. The TSS descriptor also contains access permissions and type information
            // Think of it as: "Hey CPU, our emergency stacks are stored in THIS memory location"
            let tss_selector=gdt.append(Descriptor::tss_segment(&TSS));
            (gdt, Selectors{code_selector,tss_selector})
        };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}
pub fn init() {
    // This tells the CPU "forget your old GDT, use this new one instead"
    // The GDT contains our code descriptor and TSS descriptor
    // After this, the CPU knows about our descriptors but isn't using them yet
    GDT.0.load();

    unsafe {
        // Even though we loaded the GDT, the CS register still points to the old code segment
        // We must explicitly tell the CPU: "use the NEW code segment from our GDT"
        // This ensures the CPU is using our kernel code segment with proper privilege levels
        // Without this, we'd still be using the bootloader's code segment, which might
        // have different permissions or configurations that could cause issues
        CS::set_reg(GDT.1.code_selector);

        // This is the most critical step for our double fault handling!
        // We're telling the CPU: "when you need emergency stacks, look in THIS TSS"
        // The CPU stores the TSS selector in a special register (TR - Task Register)
        // Now when a double fault occurs, the CPU will:
        // 1. Look at the TR register to find our TSS
        // 2. Find IST[0] in our TSS (which we set up earlier)
        // 3. Switch to that safe stack BEFORE pushing any exception info
        // 4. This prevents the triple fault because we're using a good stack
        // Without this step, our IST setup would be completely useless!
        load_tss(GDT.1.tss_selector);
    }
}
