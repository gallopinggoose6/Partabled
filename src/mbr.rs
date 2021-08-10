// Includes structs and APIs for parsing and writing MBR-based disks and partition tables

//use uefi::prelude::*;
use crate::alloc::vec::Vec;
use core::convert::TryInto;
use core::mem;

/// the signature of the MBR to ensure we actually read stuff
const MBR_SIG: [u8; 2] = [0x55, 0xaa];


/// defines the types of MBR partitions
/// note: we only include partitions we support here
#[derive(Copy,Clone,PartialEq)]
enum MbrPartTypes {
    Empty,              // id 0x00
    NTFS,               // id 0x07, 0x27 (ntfs recovery)
    Fat32,              // id 0x0b, 0x0c, 0x1c
    LinuxSwap,          // id 0x82    
    LinuxFS,            // id 0x83
    EFIProtectiveMBR,   // id 0xee
    EFISystem,          // id 0xef
    //Fat12,            // ids 0x01, 0x11, 0x61
    //Fat16,            // ids 0x04, 0x06, 0x0e, 0x14, 0x1e, 0x24, 0x56, 0x64, 0x66, 0x74, 0x76
    //LogicalFat,       // id 0x08 (note overlaps with OS/2, AIX boot, QNY)
}

/// defines an MBR partition
#[derive(Copy,Clone)]
struct MbrPartition {
    active:         bool,
    // 0x1-> head, 
    // 0x2-> bits 5-0 -> sector, bits 7-6 -> high bits of cylinder (i.e. 9-8 of cylinder)
    // 0x3-> bits 7-0 -> sector 7-0
    chs_start:      [u8; 3],  
    chs_end:        [u8; 3],
    part_type:      MbrPartTypes,
    lba_start:      u64,
    num_sectors:    u64
}


/// defines our MBR structure
struct MBR {
    partitions: Vec<MbrPartition>,
}


////////////////////// PARTITION FUNCTIONS /////////////////////////////
impl MbrPartition {
    /// create a new MbrPartition
    fn new(partition_buffer: [u8; 16]) -> Self {
        // see if partition is "active"
        let active = match (partition_buffer[0] & 0x80) {
            0 => false,
            _ => true
        };
        // get the cylinder-head-sector start and end
        let chs_start:  [u8; 3] = partition_buffer[1..4]
                                    .try_into().unwrap();
        let chs_end:    [u8; 3] = partition_buffer[5..8]
                                    .try_into().unwrap();

        // determine the partition type
        let part_type = match partition_buffer[4] {
            0x00 => MbrPartTypes::Empty,
            0x07 | 0x27 => MbrPartTypes::NTFS,
            0x0b | 0x0c | 0x1c => MbrPartTypes::Fat32,
            0x82 => MbrPartTypes::LinuxSwap,
            0x83 => MbrPartTypes::LinuxFS,
            0xee => MbrPartTypes::EFIProtectiveMBR,
            0xef => MbrPartTypes::EFISystem,
            _ => panic!("Unknown partition type! {}", partition_buffer[4])
        };

        // get the LBA and sector counts
        let lba_start = u64::from_ne_bytes(
            partition_buffer[8..12]
            .try_into().unwrap()
        );
        let num_sectors = u64::from_ne_bytes(
            partition_buffer[12..16]
            .try_into().unwrap()
        );

        MbrPartition {
            active,
            chs_start,
            chs_end,
            part_type,
            lba_start,
            num_sectors
        }
    }

    /// returns the status of the partition
    pub fn active(&self) -> bool {
        self.active
    }

    /// returns the partition's type 
    pub fn part_type(&self) -> MbrPartTypes {
        self.part_type
    }

    /// returns the lba start of the partition 
    pub fn lba_start(&self) -> u64 {
        self.lba_start
    }

    /// returns the number of sectors in the partition
    pub fn num_sectors(&self) -> u64 {
        self.num_sectors
    }
}


////////////////////// MBR MAIN FUNCTIONS ////////////////////////
impl MBR {
    /// creates a bew MBR structure 
    pub fn new(bootsector: Vec<u8>) -> Self {
        // make sure the partition actually has the MBR signature
        assert_eq!(bootsector[510..511], MBR_SIG, "Boot sector is not an MBR!");

        // create our variables
        let mut partitions: Vec<MbrPartition> = Vec::new();

        // This is simply a reminder for offsets  
        // bootcode = bootsector[0..445];
        let p1: [u8; 16] = bootsector[446..462].try_into().unwrap();
        let p2: [u8; 16] = bootsector[462..478].try_into().unwrap();
        let p3: [u8; 16] = bootsector[478..494].try_into().unwrap();
        let p4: [u8; 16] = bootsector[494..510].try_into().unwrap();

        // push the partitions to the vector
        partitions.push(MbrPartition::new(p1));
        partitions.push(MbrPartition::new(p2));
        partitions.push(MbrPartition::new(p3));
        partitions.push(MbrPartition::new(p4));

        MBR {
            partitions
        }


    }
}