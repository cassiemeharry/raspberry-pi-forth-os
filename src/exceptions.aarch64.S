#include "exceptions.aarch64.h"

.section ".text.vector_table"

.macro ventry label
    .p2align    7
.globl ventry_\label
ventry_\label:
    b       \label
.endm

.macro handle_invalid_entry type
    .p2align 0
    kernel_entry
    mov     x0, #\type
    mrs     x1, esr_el1
    mrs     x2, elr_el1
    b      show_invalid_entry_message
.endm

.macro kernel_entry
    stp     x0, x1, [sp, #-16]!
    stp     x2, x3, [sp, #-16]!
    stp     x4, x5, [sp, #-16]!
    stp     x6, x7, [sp, #-16]!
    stp     x8, x9, [sp, #-16]!
    stp     x10, x11, [sp, #-16]!
    stp     x12, x13, [sp, #-16]!
    stp     x14, x15, [sp, #-16]!
    stp     x16, x17, [sp, #-16]!
    stp     x18, x19, [sp, #-16]!
    stp     x20, x21, [sp, #-16]!
    stp     x22, x23, [sp, #-16]!
    stp     x24, x25, [sp, #-16]!
    stp     x26, x27, [sp, #-16]!
    stp     x28, x29, [sp, #-16]!
    str     x30, [sp, #-8]!
.endm

.macro kernel_exit
    ldp     x0, x1, [sp], #16
    ldp     x2, x3, [sp], #16
    ldp     x4, x5, [sp], #16
    ldp     x6, x7, [sp], #16
    ldp     x8, x9, [sp], #16
    ldp     x10, x11, [sp], #16
    ldp     x12, x13, [sp], #16
    ldp     x14, x15, [sp], #16
    ldp     x16, x17, [sp], #16
    ldp     x18, x19, [sp], #16
    ldp     x20, x21, [sp], #16
    ldp     x22, x23, [sp], #16
    ldp     x24, x25, [sp], #16
    ldp     x26, x27, [sp], #16
    ldp     x28, x29, [sp], #16
    ldr     x30, [sp], #8
    eret
.endm

.p2align      11
.globl vectors
vectors:
    ventry  sync_invalid_el1t   // Synchronous EL1t
    ventry  irq_invalid_el1t    // IRQ EL1t
    ventry  fiq_invalid_el1t    // FIQ EL1t
    ventry  error_invalid_el1t  // Error EL1t

    ventry  el1_sync            // Synchronous EL1h
    ventry  el1_irq             // IRQ EL1h
    ventry  fiq_invalid_el1h    // FIQ EL1h
    ventry  error_invalid_el1h  // Error EL1h

    ventry  sync_invalid_el0_64  // Synchronous 64-bit EL0
    ventry  irq_invalid_el0_64   // IRQ 64-bit EL0
    ventry  fiq_invalid_el0_64   // FIQ 64-bit EL0
    ventry  error_invalid_el0_64 // Error 64-bit EL0

    ventry  sync_invalid_el0_32  // Synchronous 32-bit EL0
    ventry  irq_invalid_el0_32   // IRQ 32-bit EL0
    ventry  fiq_invalid_el0_32   // FIQ 32-bit EL0
    ventry  error_invalid_el0_32 // Error 32-bit EL0

.section ".text.exception_handlers"

sync_invalid_el1t:
    handle_invalid_entry SYNC_INVALID_EL1t

irq_invalid_el1t:
    handle_invalid_entry IRQ_INVALID_EL1t

fiq_invalid_el1t:
    handle_invalid_entry FIQ_INVALID_EL1t

error_invalid_el1t:
    handle_invalid_entry ERROR_INVALID_EL1t

// sync_invalid_el1h:
//     handle_invalid_entry SYNC_INVALID_EL1h

fiq_invalid_el1h:
    handle_invalid_entry FIQ_INVALID_EL1h

error_invalid_el1h:
    handle_invalid_entry ERROR_INVALID_EL1h

sync_invalid_el0_64:
    handle_invalid_entry SYNC_INVALID_EL0_64

irq_invalid_el0_64:
    handle_invalid_entry IRQ_INVALID_EL0_64

fiq_invalid_el0_64:
    handle_invalid_entry FIQ_INVALID_EL0_64

error_invalid_el0_64:
    handle_invalid_entry ERROR_INVALID_EL0_64

sync_invalid_el0_32:
    handle_invalid_entry SYNC_INVALID_EL0_32

irq_invalid_el0_32:
    handle_invalid_entry IRQ_INVALID_EL0_32

fiq_invalid_el0_32:
    handle_invalid_entry FIQ_INVALID_EL0_32

error_invalid_el0_32:
    handle_invalid_entry ERROR_INVALID_EL0_32


el1_sync:
    kernel_entry
    bl      handle_sync
    kernel_exit

el1_irq:
    kernel_entry
    bl      handle_irq
    kernel_exit
