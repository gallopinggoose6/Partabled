//! This defines our algorithm for determining what kinds of 
//! algorithmic shenanigans will be done to shift about partitions

// import the required MBR/GPT struct definitions
use crate::partitions::{
    GPT,
    GPTPartition,
    MBR
};

// import the required helper functions
use crate::helpers::{
    get_free_ram_size,
};

/*
Assuming we know the two partitions we want to swap on disk
the approach we are gonna use for swapping is as follows: 

1. Allocate as much memory (disk block aligned) as possible FOR EACH PARTITION
    - Call these two chunks PBLOCK_1 and PBLOCK_2
    - Note that the requirement is that (SIZE/2) % BLOCK_SIZE = 0
      which needs to be gueranteed so we can properly shift the blocks around
      on the disk
2. Read as many blocks from partition 1 -> PBLOCK_1
3. Read as many blocks from parititon 2 -> PBLOCK_2
4. Write PBLOCK_1 to the head of partition 2, and PBLOCK_2 to head of part1
5. Increment counters and offset, ZERO PBLOCK MEMORY, then repeat for the next offset
    - ISSUE: if the partitions are unequally sized, then program needs to be 
             able to determine how to place the partitions such that the growth 
             of a partition doesn't impact




OR WE COULD BE SMART AND JUST SWAP THE PARTITION ENTRIES ON THE DISK LOL

*/

/// swaps two GPT partitions with eachother
pub fn swap_gpt_partitions(
    st: &mut SystemTable<Boot>,
    disk: GPT,
    part_1: GPTPartition,
    part_2: GPTPartition
) -> u32 {
    
    0
}

/// Moves a GPT partition to begin at a different LBA (UNSAFE)
pub fn move_gpt_partition_safe(
    st: &mut SystemTable<Boot>,
    disk: GPT,
    target: GPTPartition,
    new_lba_start: u64
) -> u32 {
    unimplemented!();
}


/// Moves a GPT partition to begin at a different LBA 
/// 
/// Panics if `new_lba_start` overlaps with another partition's existing domain
/// If you want something that would automatically try to move the partitions 
/// to allow the move to occur, see `move_gpt_partition_unsafe`
pub fn move_gpt_partition_safe(
    st: &mut SystemTable<Boot>,
    disk: GPTDisk,
    target: GPTPartition,
    new_lba_start: u64
) -> u32 {

    // check if the partition is going to overlap with any existing partitions
    for part in disk.partitions().iter() {
        if part.part_guid() != target.part_guid() {
            // check for overlaps
            if new_lba_start < part.last_lba() && 
               new_lba_start > part.first_lba() {
                // what should we do here?
                // panicking for the moment...
                panic!("Selected partition start LBA overlaps another partition!");
            } 

        }
    }

    // save the disk length for when we update 
    // the partition information later
    let target_len = target.last_lba() - target.first_lba()

    // determine how much memory we can use and allocate that much 
    // note we save 32 MB for overheads just in case
    let curr_free_ram = get_free_ram_size(st.boot_services()) - 32*1024*1024;
    let usable_ram = curr_free_ram - (curr_free_ram % disk.blocksize);
    let mut part_1_data = vec![0u8; usable_ram as usize]; 
    
    let bs = st.boot_services();
    let handles2 = bs.find_handles::<BlockIO>()
                         .expect_success("Failed to find handles for `BlockIO`");

    // loop over all handles and see if they are for the media we want
    for handle in handles2 {
        let bi = bs.handle_protocol::<BlockIO>(handle)
                   .expect_success("Failed to get `BlockIO` protocol");
        let bi = unsafe {&* bi.get()};

        // get the variables of the media we need
        let test_media_id = bi.media().media_id();
        let blocksize = bi.media().block_size();

        // check the device's media id against the target one
        if test_media_id == disk.media_id {
            // make sure the disk is writable
            if !bi.media().is_writable() {
                panic!("Disk is not writable");
            } 

            // calculate the number of loops we should do
            let num_loops = target_len / usable_ram;
            // now that we know its not overlapping anything, 
            // shift the bytes to the new position
            for i in 0..num_loops {
                // try to read the data
                bi.read_blocks(disk.media_id, 
                    target.lba_start() + (i*usable_ram), 
                    &mut part_1_data)
                    .expect("Failed to read bytes")
                    .unwrap();

                // try to write the data to the new LBA
                bi.write_blocks(
                    disk.media_id,
                    new_lba_start + (i*usable_ram), 
                    &mut part_1_data)
                    .expect("Failed to write bytes")
                    .unwrap();
            }
        } 
    }

    

    0
}