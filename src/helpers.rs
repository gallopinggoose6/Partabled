// Includes various helper functions for ease of use
use uefi::prelude::*;
use uefi::table::boot::{
    BootServices,
    MemoryDescriptor,
};
use uefi::proto::{
    loaded_image::LoadedImage,
    device_path::DevicePath,
    media::block::BlockIO
};

use crate::alloc::vec::Vec;
use core::mem;
use core::convert::TryInto;


/// define a return type that we can use to help keep all information needed together
pub struct BootRecord {
    pub data: [u8; 512],
    pub media_id: u32
}



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
pub fn print_system_info(image: &Handle, st: &mut SystemTable<Boot>) {
    // clear the console
    st.stdout()
        .clear()
        .expect("Failed to clear screen")
        .unwrap();

    // set up aliases for boot and runtime services
    let bs = st.boot_services();
    

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
    let length = dev_handle.length();
    info!("Path: type={:?}, subtype={:?}, length={}",
        dev_handle.device_type(), dev_handle.sub_type(), length);
    
    //let disk_handle = image.device();
    
}

/// returns all disks protocol
pub fn read_all_bootsectors(st: &mut SystemTable<Boot>) -> Vec<BootRecord>{
    let bs = st.boot_services();
    let mut ret: Vec<BootRecord> = Vec::new();

    // get all handles available for BlockIO operations
    // note this code is known-working when injected to the end of the
    // uefi-test-runner's media tests 
    let handles2 = bs
        .find_handles::<BlockIO>()
        .expect_success("failed to find handles for `BlockIO`");

    for handle in handles2 {
        let bi = bs
            .handle_protocol::<BlockIO>(handle)
            .expect_success("Failed to get BlockIO protocol");

        let bi = unsafe {&* bi.get()};

        let bmedia = bi.media();
        let media_id = bmedia.media_id();
        let block_size = bmedia.block_size();
        let last_block = bmedia.last_block();
        let low_lba = bmedia.lowest_aligned_lba();

        info!("Disk size: {}", block_size as u64 * (last_block+1));
    
        // attempt to read from the buffer
        let mut buf: Vec<u8> = vec![0u8; block_size as usize];

        bi.read_blocks(media_id, low_lba, &mut buf)
            .expect("Failed to read bytes")
            .unwrap();

        info!("Successfully read bytes from media");
        info!("{:?}", &buf[0..2]);
         
        // push the data into our return vector
        let data: [u8; 512] = buf[..].try_into().unwrap();
        ret.push(
            BootRecord{
                data,
                media_id
            }
        );
    }

    ret
}
