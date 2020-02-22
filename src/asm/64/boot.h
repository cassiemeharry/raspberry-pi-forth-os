#define VA_START                0xffff000000000000

#define PAGE_SHIFT              12
#define TABLE_SHIFT             9
#define SECTION_SHIFT           (PAGE_SHIFT + TABLE_SHIFT)

#define PAGE_SIZE               (1 << PAGE_SHIFT)
#define SECTION_SIZE            (1 << SECTION_SHIFT)

#define LOW_MEMORY              (2 * SECTION_SIZE)

#define HCR_RW                  (1 << 31)
#define HCR_VALUE               HCR_RW

#define MT_DEVICE_nGnRnE        0x0
#define MT_NORMAL_NC            0x1
#define MT_DEVICE_nGnRnE_FLAGS  0x00
#define MT_NORMAL_NC_FLAGS      0x44
#define MAIR_VALUE              (MT_DEVICE_nGnRnE_FLAGS << (8 * MT_DEVICE_nGnRnE)) | (MT_NORMAL_NC_FLAGS << (8 * MT_NORMAL_NC))

#define SCR_RESERVED            (3 << 4)
#define SCR_RW                  (1 << 10)
#define SCR_NS                  (1 << 0)
#define SCR_VALUE               (SCR_RESERVED | SCR_RW | SCR_NS)

/* #define SCTLR_RESERVED          (3 << 28) | (3 << 22) | (1 << 20) | (1 << 11) */
#define SCTLR_RESERVED          (1 << 22) | (1 << 11) | (1 << 5) | (1 << 4) | (1 << 3)
#define SCTLR_SPAN              (1 << 23)
#define SCTLR_TRAP_EL0_WFE      (1 << 18)
#define SCTLR_TRAP_EL0_WFI      (1 << 16)
#define SCTLR_EE_LITTLE_ENDIAN  (0 << 25)
#define SCTLR_EOE_LITTLE_ENDIAN (0 << 24)
#define SCTLR_I_CACHE_DISABLED  (0 << 12)
#define SCTLR_D_CACHE_DISABLED  (0 << 2)
#define SCTLR_MMU_DISABLED      (0 << 0)
#define SCTLR_MMU_ENABLED       (1 << 0)
#define SCTLR_INIT_MMU_DISABLED    (SCTLR_RESERVED | SCTLR_SPAN | SCTLR_TRAP_EL0_WFE | SCTLR_TRAP_EL0_WFI | SCTLR_EE_LITTLE_ENDIAN | SCTLR_I_CACHE_DISABLED | SCTLR_D_CACHE_DISABLED | SCTLR_MMU_DISABLED)

#define SPSR_MASK_ALL           (7 << 6)
#define SPSR_EL1h               (5 << 0)
#define SPSR_VALUE              (SPSR_MASK_ALL | SPSR_EL1h)

#define TCR_T0SZ                (64 - 48)
#define TCR_T1SZ                ((64 - 48) << 16)
#define TCR_TG0_4K              (0 << 14)
#define TCR_TG1_4K              (2 << 30)
#define TCR_VALUE               (TCR_T0SZ | TCR_T1SZ | TCR_TG0_4K | TCR_TG1_4K)
