// Includes various helper functions for ease of use
use uefi::prelude::*;
use uefi::table::boot::{
    BootServices,
    AllocateType,
    MemoryDescriptor,
    MemoryType
};
use crate::alloc::vec::Vec;
use core::mem;

/// helps determine the total free space in RAM
pub fn get_free_ram_size(services: &BootServices) -> u64 {
    // get the memory size of the current memory map
    // note this is simply a guess, so we add a few more descriptors to it 
    // to make sure we have enough memory to store the structure
    let mm_size = services.memory_map_size() + 8 * mem::size_of::<MemoryDescriptor>();

    // get a vector so we can store data in it
    let mut buf = Vec::with_capacity(mm_size);
    unsafe {buf.set_len(mm_size);}

    let (_key, desc_iter) = services.memory_map(&mut buf).expect_success("Failed to retrieve memory map");
    
    let mut mem_size = 0u64;
    // loop over each descriptor and count its size
    for desc in desc_iter {
        mem_size += desc.page_count;
    }

    // return the number of pages
    mem_size
}
