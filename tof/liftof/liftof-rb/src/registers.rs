//! Registers of the DRS4 are accessed through
//! the sytem ram (Addr8). It is a 32bit system, 
//! so the address format is u32.
//! _ a note here _ : Each register is 32bit. This means for 
//! the Addr8 (8 refers to bits) a register occupies 4 bytes, 
//! so a new register will be the previous register + 4.
//! If the register is the same as another, then the register
//! holds different fields for the different bits in the register.
//!
//! Please refere to 
//! https://gitlab.com/ucla-gaps-tof/firmware/-/blob/develop/regmap/rb_address_table.org
//! DRS4 readout



//========== DRS4 Registers =============
//
//=======================================

pub const ROI_MODE         : u32 =   0x40;    //00x1  Set to 1 to enable Region of Interest Readout
pub const BUSY             : u32 =   0x40;    //1 DRS is busy
pub const ADC_LATENCY      : u32 =   0x40;    //[9:4] rw  0x9 Latency from first sr clock to when ADC data should be valid
pub const SAMPLE_COUNT     : u32 =   0x40;    //[21:12]   rw  0x3FF   Number of samples to read out (0 to 1023)
pub const EN_SPIKE_REMOVAL : u32 =   0x40;    //22    rw  0x1 set 1 to enable spike removal

pub const FORCE_TRIG       : u32 =   0x100;   // Write 1 to set forced trigger mode

pub const DRS_CONFIGURE    : u32 = 0x50;    // Write 1 to configure the DRS. Should be done before data taking
pub const DRS_START        : u32 = 0x48; // Write 1 to take the state machine out of idle mode
pub const DRS_REINIT       : u32 = 0x4c; // Write 1 to reinitialize DRS state machine (restores to idle state) 

pub const DMA_CLEAR : u32 = 0x6c; // [0] Write 1 to clear the DMA memory (write zeroes)

pub const DRS_RESET : u32 = 0x54; // WRite 1 to completely reset the DRS state machine logic
pub const DAQ_RESET : u32 = 0x58; // Write 1 to completely reset the DAQ state machine logic
pub const DMA_RESET : u32 = 0x5c; // Write 1 to completely reset the DMA state machine logic

// channel mask is 8 bit, the register contains also 
// the channel 9 auto bit mask
pub const READOUT_MASK : u32 = 0x44; // [8:0] 8 bit mask, set a bit to 1 to enable readout of that channel.
                                 // 9th is auto-read if any channel is enabled and AUTO_9TH_CHANNEL set to 1

pub const TRIGGER_ENABLE : u32 = 0x11c;  // Write 0 to stop all triggers, 1 to enable triggering

pub const WRITE_EVENTFRAGMENT : u32 = 0xc4;
pub const TRIG_GEN_RATE       : u32 = 0x164;

//=================DMA==================================
// (direct memory access)
//======================================================

/// RAM management - there are two regions in memory, mapped
/// to /dev/uio1 and /dev/uio2 which hold the blob data, 
/// denoted as ram buffers a and b
pub const RAM_A_OCC_RST    :u32 =  0x400;//[0] Sets RAM buffer a counter to 0
pub const RAM_B_OCC_RST    :u32 =  0x404;//[0] Sets RAM buffer b counter to 0
pub const RAM_A_OCCUPANCY  :u32 =  0x408;//[31:0] RAM buffer a occupancy
pub const RAM_B_OCCUPANCY  :u32 =  0x40c;//[31:0] RAM buffer b occupancy
pub const DMA_POINTER      :u32 =  0x410;//[31:0] DMA controller pointer
pub const TOGGLE_RAM       :u32 =  0x414;//[0] Write 1 to switch the dma buffer to the other half

/// DRS trigger
pub const MT_TRIGGER_MODE            : u32 = 0x114; //1 to use the MT as the source of the trigger


/// DRS counters

pub const CNT_SEM_CORRECTION         : u32 = 0x140; //[15:0] Number of Single Event Errors corrected by the scrubber
pub const CNT_SEM_UNCORRECTABLE      : u32 = 0x144; //[19:16] Number of Critical Single Event Errors (uncorrectable by scrubber)
pub const CNT_READOUTS_COMPLETED     : u32 = 0x148; // [31:0] Number of readouts completed since reset
pub const CNT_DMA_READOUTS_COMPLETED : u32 = 0x14c; //[31:0] Number of readouts completed since reset
pub const CNT_LOST_EVENT             : u32 = 0x150; //[31:16] Number of trigger lost due to deadtime
pub const CNT_EVENT                  : u32 = 0x154; //[31:0] Number of triggers received
pub const TRIGGER_RATE               : u32 = 0x158; //[31:0] Rate of triggers in Hz
pub const LOST_TRIGGER_RATE          : u32 = 0x15c; //[31:0]Rate of lost triggers in Hz  
pub const CNT_RESET                  : u32 = 0x160; //[0]Reset the counters

/// Device DNA (identifier)
/// it is split in 2 32-bit words, since the whole 
/// thing is 64 bit
pub const DNA_LSBS : u32 = 0x80;    //[31:0]    Device DNA [31:0]
pub const DNA_MSBS : u32 = 0x84;    //[24:0]    Device DNA [56:32]

// FPGA
pub const BOARD_ID : u32 = 0xa8;    //[7:0]	    Board ID Number

// SOFT RESET
pub const SOFT_RESET : u32 = 0x70;
pub const SOFT_RESET_DONE : u32 = 0x74;

// MT EVENT REGISTERS
pub const MT_EVENT_CNT : u32 = 0x120;
pub const MT_TRIG_RATE : u32 = 0x124;
pub const MT_LINK_ID   : u32 = 0x104;
