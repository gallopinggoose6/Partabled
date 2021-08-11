// Includes structs and APIs for handing of the GPT partition table format
use uefi::prelude::*;

const EFI_SIG: [u8; 8] = *b"EFI PART";

/// define our GPT Partition Table header
pub struct GPTHeader {
    // [0..8] -> EFI SIG
    revision: u32, // [8..12]
    header_sz: u32, // [12..16]
    crc32: u32, // [16..20] note to check this, assume this is filled with zero
    // [20..24] -> RESERVED (must be zeroes)
    curr_lba: u64, // [24..32]
    backup_lba: u64, // [32..40]
    first_lba: u64, // [40..48]
    last_lba: u64, // [48..56]
    guid: [u8; 16], // [56..72]
    lba_part_entries: u64, // [72, 80]
    num_partitions: u32, // [80..84]
    part_size: u32, // [84..88]
    part_crc32: u32 // [88..92]
    // [92] -> RESERVED, rest are zeroes
}

/// define our GPT Partition Entry struct 
pub struct GPTPartition {
    part_type_guid: [u8;16], // [0..16]
    part_guid: [u8; 16], // [16..32]
    first_lba: u64, // [32..40]
    last_lba: u64, // [40..48]
    attr_flags: u64, // [48..56]
    part_name: [u8; 72] // [56..128] note this is stored using UTF-16 LE encoding...
}

/// define out GPT struct, which will take an EFI disk to parse
pub struct GPT {
    disk_num: u16
}

/// define our functions for the GPT struct so we can use it later on
impl GPT {
    fn new() -> Self {
        GPT {
            disk_num: 0
        }
    }

    /// checks to see if a disk has a legacy or protective MBR
    fn check_mbr(&self) -> Result<(), u32> {
        Ok(())
    }
}
