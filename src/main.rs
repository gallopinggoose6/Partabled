#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

// Keep this line to ensure the `mem*` functions are linked in.
extern crate rlibc;

use core::mem;
use uefi::prelude::*;
use uefi::table::boot::MemoryDescriptor;
use crate::alloc::vec::Vec;



// include our local files too
mod gpt;
mod mbr;
mod ext4;
mod fat32;
mod helpers;




#[entry]
fn efi_main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    // initialize the crap
    uefi_services::init(&mut st).expect_success("Failed to initialized system table stuff");
    
    // make sure we disable the watchdog so the firmware doesn't interrupt our program
    st.boot_services().set_watchdog_timer(0, 0xffffffffu64, None).expect("Failed to disable watchdog");

    // print version information
    helpers::print_system_info(&image, &mut st);

    // get the bootsectors of the various blockio devices
    let bootsectors: Vec<[u8; 512]> = helpers::read_all_bootsectors(&mut st);

    // try to parse the MBRs of each bootsector
    let mut bootsecs: Vec<mbr::MBR> = Vec::new();
    for bootsec in bootsectors.iter() {
        bootsecs.push(mbr::MBR::new(*bootsec));
    }

    // wait a bit, then shutdown
    st.boot_services().stall(1_000_000);


    shutdown(image, st);
}

/// shutdown the system
fn shutdown(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    use uefi::table::runtime::ResetType;

    // Get our text output back.
    //st.stdout().reset(false).unwrap_success();

    // Inform the user we are done
    info!("Testing complete, shutting down in 3 seconds...");
    st.boot_services().stall(3_000_000);
    
    // Exit boot services as a proof that it works :)
    let max_mmap_size =
        st.boot_services().memory_map_size() + 8 * mem::size_of::<MemoryDescriptor>();
    let mut mmap_storage = vec![0; max_mmap_size].into_boxed_slice();
    let (st, _iter) = st
        .exit_boot_services(image, &mut mmap_storage[..])
        .expect_success("Failed to exit boot services");

    // Shut down the system
    let rt = unsafe { st.runtime_services() };
    rt.reset(ResetType::Shutdown, Status::SUCCESS, None);
}
