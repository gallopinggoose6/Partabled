// Includes various helper functions for ease of use
use uefi::prelude::*;
use uefi::{Result, Status};
use uefi::table::boot::{
    BootServices,
    MemoryDescriptor,
    MemoryType,
    AllocateType
};
use uefi::proto::{
    Protocol,
    loaded_image::LoadedImage,
    device_path::DevicePath,
    media::block::BlockIO
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


/// function that prints system information
pub fn print_system_info(image: &Handle, st: &SystemTable<Boot>) {
    // set up aliases for boot and runtime services
    let bs = st.boot_services();
    let rt = st.runtime_services();

    // clear the console
    st.stdout().clear().expect("Failed to clear screen");


    // print the firmware version to the console
    let firmware_vendor = st.firmware_vendor();
    let firmware_revision = st.firmware_revision();
    info!("Running on firmware: {} ({} major, {} minor)", firmware_vendor, 
            firmware_revision.major(), firmware_revision.minor());

    // determine the number of pages and bytes available on the system
    let ram_size = get_free_ram_size(st.boot_services());
    info!("Determined RAM size: {} pages ({} bytes)", ram_size, ram_size * 4096);


    // attempt to get the handle for the device the current image is stored on
    let img_proto = bs.handle_protocol::<LoadedImage>(*image)
                  .expect_success("Failed to handle loaded image protocol");
    let img_loaded = unsafe {&*img_proto.get()};

    // now that we have fetched the loaded image, try to get the device path for it
    let dev_proto = bs.handle_protocol::<DevicePath>(img_loaded.device())
                      .expect_success("Failed to get the device's protocol");
    let dev_handle = unsafe {&*dev_proto.get()};

    // now print the device information
    let length = ((dev_handle.length[0] as u16) << 8) | dev_handle.length[1] as u16;
    info!("Path: type={:?}, subtype={:?}, length={}",
        dev_handle.device_type, dev_handle.sub_type, length);
    
    //let disk_handle = image.device();
    
}

/// returns all disks protocol
pub fn get_disk_protos(bs: &BootServices) -> Vec<u8>{
    // see if the BlockIO protocol is supported on the device
    if let Ok(block_io) = bs.locate_protocol::<BlockIO>() {
        let block_io = block_io.expect("Failed to open block device");
        let block_io = unsafe {&* block_io.get()};

        // try to get the media information of the handle
        let b_info = block_io.media();

        // print the media info
        info!(
            "Found media: id={}, removable={}, present={}, bs={}, ro={}",
            b_info.media_id(), 
            b_info.is_removable_media(), 
            b_info.is_media_preset(), 
            b_info.block_size(), 
            b_info.is_read_only()
        );

        info!("Allignment information: {}", b_info.io_align());

        // save what we need to read bytes
        let mid = b_info.media_id();
        let mut buffer = vec![0u8;512];

        

        ////// BROKEN HERE //////
        // save what we need to read a block
        let mut mid = b_info.media_id();
        let first_lba = b_info.lowest_aligned_lba();
        let mut buffer = vec![0u8; 512];
        let alloc_t = AllocateType::AnyPages;
        let mut pg = bs.allocate_pages(alloc_t, MemoryType::LOADER_DATA, 1)
                       .expect_success("Failed to allocate pool");


        
        let mut buff = unsafe { &mut *(pg as *mut [u8; 4096])};

        info!("Attempting to read block..");
        // try to read the block's info
        loop {
            info!("looping...");
            match block_io.read_blocks(mid, 0, buff) {
                Ok(_) => {
                    info!("Success!");
                    break
                },
                Err(e) => {
                    info!("Caught Error");
                    match e.status() {
                    Status::DEVICE_ERROR => {
                        info!("Device error detected");
                        break;
                    },
                    Status::NO_MEDIA => {
                        info!("No media for that ID detected");
                        break;
                    },
                    Status::MEDIA_CHANGED => {
                        info!("Detected media ID change, updating...");
                        mid = b_info.media_id();
                    },
                    Status::BAD_BUFFER_SIZE => {
                        info!("Bad buffer size, resizing...");
                        let min_size = buffer.len()+1024; // temporary fix until we can actually get the new size back
                        buffer.resize(min_size, 0);
                    },
                    Status::INVALID_PARAMETER => {
                        info!("Invalid LBA/buffer allignment");
                        break;
                    },
                    Status(_) => {
                        info!("Unexpected error detected");
                        break;
                    }
                    }
                }
            };
            info!("Finished loop");
        }
                   

    } else {
        warn!("BlockIO Protocol is not available");
    }

    vec![0u8;2]
}
