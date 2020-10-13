// AArch64 mode

#include "boot.h"

// To keep this in the first portion of the binary.
.section ".text.boot"

// Make _start global.
.globl _start

.globl _start
_start:
    mrs     x0, mpidr_el1
    and     x0, x0, #0xFF       // Check processor id
    cbz     x0, reset           // Hang for all non-primary CPU

proc_hang:
    wfi
    b       proc_hang

reset:
    mrs     x0, CurrentEL
    lsr     x0, x0, 2

    ldr     x0, =(SCTLR_INIT_MMU_DISABLED)
    msr     sctlr_el1, x0

    ldr     x0, =HCR_VALUE
    msr     hcr_el2, x0

    ldr     x0, =SCR_VALUE
    msr     scr_el3, x0

    ldr     x0, =SPSR_VALUE
    msr     spsr_el3, x0

    adr     x0, el1_entry
    msr     elr_el3, x0

    eret

el1_entry:
    // Set a temporary stack pointer
    ldr     x0, =(0xFFFF_0000_0000_0000 | (1 << 28))
    mov     sp, x0
    bl      __memory_init
    b       kernel_main

;;     ldr     x2, =kernel_main

;;     mov     x0, #VA_START
;;     add     sp, x0, #LOW_MEMORY

.globl enable_mmu
enable_mmu:
    mrs     x0, sctlr_el1
    orr     x0, x0, #SCTLR_MMU_ENABLED
    msr     sctlr_el1, x0
    ret

;;     br      x2

.globl delay
delay:
    subs    x0, x0, #1
    bne     delay
    ret

err_hang_msg:
    .asciz "System fatal error"

.globl err_hang
.p2align 3
err_hang:
    ldr     x0, =err_hang_msg
    bl      puts
1:  wfe
    b       1b
