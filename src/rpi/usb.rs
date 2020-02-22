// This is based on https://github.com/LdB-ECM/Raspberry-Pi/blob/33b176202b5ee12e19ed8757526190e62efb2975/Arm32_64_USB/rpi-usb.c

#![allow(non_camel_case_types)]

use bitflags::bitflags;

// These constants must be within [16, 32768]
const FIFO_SIZE_RECEIVE: usize = 20480;
const FIFO_SIZE_NON_PERIODIC: usize = 20480;
const FIFO_SIZE_PERIODIC: usize = 20480;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
enum CoreFifoFlush {
    NonPeriodic = 0,
    Periodic1 = 1,
    Periodic2 = 2,
    Periodic3 = 3,
    Periodic4 = 4,
    Periodic5 = 5,
    Periodic6 = 6,
    Periodic7 = 7,
    Periodic8 = 8,
    Periodic9 = 9,
    Periodic10 = 10,
    Periodic11 = 11,
    Periodic12 = 12,
    Periodic13 = 13,
    Periodic14 = 14,
    Periodic15 = 15,
    All = 16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct FifoSize {
    start_address: u16,
    depth: u16,
}

bitflags! {
    struct ChannelInterrupts: u32 {
        const TRANSFER_COMPLETE = 1 << 0;
        const HALT = 1 << 1;
        const AHB_ERROR = 1 << 2;
        const STALL = 1 << 3;
        const NEG_ACK = 1 << 4;
        const ACK = 1 << 5;
        const NOT_YET = 1 << 6;
        const TRANSACTION_ERROR = 1 << 7;
        const BABBLE_ERROR = 1 << 8;
        const FRAME_OVERRUN = 1 << 9;
        const DATA_TOGGLE_ERROR = 1 << 10;
        const BUFFER_NOT_AVAILABLE = 1 << 11;
        const EXCESSIVE_TRANSMISSION = 1 << 12;
        const FRAME_LIST_ROLLOVER = 1 << 13;
    }
}

bitflags! {
    struct CoreOtgControl: u32 {
        const SESREQSCS = 1 << 0;
        const SESREQ = 1 << 1;
        const VBVALIDOVEN = 1 << 2;
        const VBVALIDOVVAL = 1 << 3;
        const AVALIDOVEN = 1 << 4;
        const AVALIDOVVAL = 1 << 5;
        const BVALIDOVEN = 1 << 6;
        const BVALIDOVVAL = 1 << 7;
        const HSTNEGSCS = 1 << 8;
        const HNPREQ = 1 << 9;
        const HOSTSETHNPENABLE = 1 << 10;
        const DEVHNPEN = 1 << 11;
        // bits 12-15 are reserved
        const CONIDSTS = 1 << 16;
        const DBNCTIME = 1 << 17;
        const ASESSIONVALID = 1 << 18;
        const BSESSIONVALID = 1 << 19;
        const OTG_VERSION = 1 << 20;
        // bit 21 is reserved
        const MULTIVALIDBC_MASK = 0b0001_1111 << 22;
        const CHIRPEN = 1 << 27;
    }
}

bitflags! {
    struct CoreOtgInterrupt: u32 {
        const SESSION_END_DETECTED = 1 << 2;
        const SESSION_REQUEST_SUCCESS_STATUS_CHANGE = 1 << 8;
        const HOST_NEGOTIATION_SUCCESS_STATUS_CHANGE = 1 << 9;
        const HOST_NEGOTIATION_DETECTED = 1 << 17;
        const A_DEVICE_TIMEOUT_CHANGE = 1 << 18;
        const DEBOUNCE_DONE = 1 << 19;
    }
}

bitflags! {
    struct CoreAhb: u32 {
        const INTERRUPT_ENABLE = 1 << 0;

        const AXI_BURST_LENGTH_1 = 3 << 1;
        const AXI_BURST_LENGTH_2 = 2 << 1;
        const AXI_BURST_LENGTH_3 = 1 << 1;
        const AXI_BURST_LENGTH_4 = 0 << 1;
        const AXI_BURST_LENGTH__MASK = 0b0011 << 1;

        const WAIT_FOR_AXI_WRITES = 1 << 4;
        const DMA_ENABLE = 1 << 5;

        const TRANSFER_EMPTY_LEVEL_HALF = 0 << 7;
        const TRANSFER_EMPTY_LEVEL_EMPTY = 1 << 7;
        const TRANSFER_EMPTY_LEVEL__MASK = 0b0001 << 7;

        const PERIODIC_TRANSFER_EMPTY_LEVEL_HALF = 0 << 8;
        const PERIODIC_TRANSFER_EMPTY_LEVEL_EMPTY = 1 << 8;
        const PERIODIC_TRANSFER_EMPTY_LEVEL__MASK = 0b0001 << 8;

        const REMEMSUPP = 1 << 21;
        const NOTIALLDMAWRIT = 1 << 22;

        const DMA_REMAINDER_MODE_INCREMENTAL = 0 << 23;
        const DMA_REMAINDER_MODE_SINGLE = 1 << 23;
        const DMA_REMAINDER_MODE__MASK = 0b0001 << 23;
    }
}

bitflags! {
    struct UsbControl: u32 {
        const TOUTCAL__MASK = 0b0111 << 0;
        const PHY_INTERFACE = 1 << 3;

        const MODE_SELECT_ULPI = 1 << 4;
        const MODE_SELECT_UTMI = 0 << 4;
        const MODE_SELECT__MASK = 0b0001 << 4;

        const FSINTF = 1 << 5;
        const PHYSEL = 1 << 6;
        const DDRSEL = 1 << 7;
        const SRP_CAPABLE = 1 << 8;
        const HNP_CAPABLE = 1 << 9;
        const USBTRDTIM__MASK = 0b1111 << 10;
        // bit 14 is reserved
        const PHY_LPM_CLK_SEL = 1 << 15;
        const OTGUTMIFSSEL = 1 << 16;
        const ULPI_FSLS = 1 << 17;
        const ULPI_AUTO_RES = 1 << 18;
        const ULPI_CLK_SUS_M = 1 << 19;
        const ULPI_DRIVE_EXTERNAL_VBUS = 1 << 20;
        const ULPI_INT_VBUS_INDICATOR = 1 << 21;
        const TS_DLINE_PULSE_ENABLE = 1 << 22;
        const INDICATOR_COMPLEMENT = 1 << 23;
        const INDICATOR_PASS_THROUGH = 1 << 24;
        const ULPI_INT_PROT_DIS = 1 << 25;
        const IC_USB_CAPABLE = 1 << 26;
        const IC_TRAFFIC_PULL_REMOVE = 1 << 27;
        const TX_END_DELAY = 1 << 28;
        const FORCE_HOST_MODE = 1 << 29;
        const FORCE_DEV_MODE = 1 << 30;
        // bit 31 is reserved
    }
}

bitflags! {
    struct CoreReset: u32 {
        const CORE_SOFT = 1 << 0;
        const HCLK_SOFT = 1 << 1;
        const HOST_FRAME_COUNTER = 1 << 2;
        const IN_TOKEN_QUEUE_FLUSH = 1 << 3;
        const RECEIVE_FIFO_FLUSH = 1 << 4;
        const TRANSMIT_FIFO_FLUSH = 1 << 5;
        const TRANSMIT_FIFO_FLUSH_NUMBER__MASK = 0b0001_1111 << 6;
        // bits 11-29 are reserved
        const DMA_REQUEST_SIGNAL = 1 << 30;
        const AHB_MASTER_IDLE = 1 << 31;
    }
}

bitflags! {
    struct CoreInterrupts: u32 {
        const CURRENT_MODE = 1 << 0;
        const MODE_MISMATCH = 1 << 1;
        const OTG = 1 << 2;
        const DMA_START_OF_FRAME = 1 << 3;
        const RECEIVE_STATUS_LEVEL = 1 << 4;
        const NP_TRANSMIT_FIFO_EMPTY = 1 << 5;
        const GINNAKEFF = 1 << 6;
        const GOUTNAKEFF = 1 << 7;
        const ULPICK = 1 << 8;
        const I2C = 1 << 9;
        const EARLY_SUSPEND = 1 << 10;
        const USB_SUSPEND = 1 << 11;
        const USB_RESET = 1 << 12;
        const ENUMERATION_DONE = 1 << 13;
        const ISOCHRONOUS_OUT_DROP = 1 << 14;
        const EOPFRAME = 1 << 15;
        const RESTORE_DONE = 1 << 16;
        const END_POINT_MISMATCH = 1 << 17;
        const IN_END_POINT = 1 << 18;
        const OUT_END_POINT = 1 << 19;
        const INCOMPLETE_ISOCHRONOUS_IN = 1 << 20;
        const INCOMPLETE_ISOCHRONOUS_OUT = 1 << 21;
        const FETSETUP = 1 << 22;
        const RESET_DETECT = 1 << 23;
        const PORT = 1 << 24;
        const HOST_CHANNEL = 1 << 25;
        const HP_TRANSMIT_FIFO_EMPTY = 1 << 26;
        const LOW_POWER_MODE_TRANSMIT_RECEIVED = 1 << 27;
        const CONNECTION_ID_STATUS_CHANGE = 1 << 28;
        const DISCONNECT = 1 << 29;
        const SESSION_REQUEST = 1 << 30;
        const WAKEUP = 1 << 31;
    }
}

bitflags! {
    struct NonPeriodicFifoStatus: u32 {
        const SPACE_AVAILABLE__MASK = 0xFFFF << 0;
        const QUEUE_SPACE_AVAILABLE__MASK = 0x00FF << 16;
        const TERMINATE = 1 << 24;

        const TOKEN_TYPE_IN_OUT = 0 << 25;
        const TOKEN_TYPE_ZERO_LENGTH_OUT = 1 << 25;
        const TOKEN_TYPE_PING_COMPLETE_SPLIT = 2 << 25;
        const TOKEN_TYPE_CHANNEL_HALT = 3 << 25;
        const TOKEN_TYPE__MASK = 0b0011 << 25;

        const CHANNEL__MASK = 0b1111 << 27;
        const ODD = 1 << 31;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CoreNonPeriodicInfo {
    size: FifoSize,
    status: NonPeriodicFifoStatus,
}

bitflags! {
    struct CoreHardware_Direction: u32 {
        const DIRECTION_0__MASK = 0b0011 << 0;
        const DIRECTION_1__MASK = 0b0011 << 2;
        const DIRECTION_2__MASK = 0b0011 << 4;
        const DIRECTION_3__MASK = 0b0011 << 6;
        const DIRECTION_4__MASK = 0b0011 << 8;
        const DIRECTION_5__MASK = 0b0011 << 10;
        const DIRECTION_6__MASK = 0b0011 << 12;
        const DIRECTION_7__MASK = 0b0011 << 14;
        const DIRECTION_8__MASK = 0b0011 << 16;
        const DIRECTION_9__MASK = 0b0011 << 18;
        const DIRECTION_10__MASK = 0b0011 << 20;
        const DIRECTION_11__MASK = 0b0011 << 22;
        const DIRECTION_12__MASK = 0b0011 << 24;
        const DIRECTION_13__MASK = 0b0011 << 26;
        const DIRECTION_14__MASK = 0b0011 << 28;
        const DIRECTION_15__MASK = 0b0011 << 30;
    }
}

// bitflags! {
//     struct CoreHardware_Info1: u32 {
//     }
// }
