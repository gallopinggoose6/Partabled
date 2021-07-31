// Includes various helper functions for ease of use
use uefi::table::boot::BootServices;

/// helps determine the total free space in RAM
pub fn get_free_ram_size(services: &BootServices) -> usize {
    services.memory_map_size()
}
