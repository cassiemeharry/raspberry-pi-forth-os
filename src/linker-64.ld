ENTRY(_start)

SECTIONS
{
    . = 0x0000;
    /* . = 0xffff000000000000; */
    __start = .;
    .text.boot : { *(.text.boot) }
    . = ALIGN(0x00001000);
    /* user_begin = .; */
    /* .text.user : { build/user* (.text) } */
    /* .rodata.user : { build/user* (.rodata) } */
    /* .data.user : { build/user* (.data) } */
    /* .bss.user : { build/user* (.bss) } */
    /* user_end = .; */
    .text :  { *(.text) }
    .rodata : { *(.rodata) }
    .data : { *(.data) }
    . = ALIGN(0x8);
    __bss_start = .;
    .bss : { *(.bss*) }
    . = ALIGN(0x8);
    __bss_end = .;
    __bss_size = __bss_end - __bss_start;
    /*
    . = 0x83000;
    .data.pgd : {
        __page_table_global = .;
        . += (1 << 12);
        __page_tables_upper = .;
        . += (2 * (1 << 12));
        __page_tables_middle = .;
        . += (4 * (1 << 12));
        __page_tables_bottom = .;
        . += (8 * (1 << 12));
        __page_tables_end = .;
    }
    */
    . = ALIGN(4096);
    __end = .;
}
