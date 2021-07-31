// Includes structs and APIs for handing of the GPT partition table format
use uefi;

/// define out GPT struct, which will take an EFI disk to parse
struct GPT {
    disk_num: u16
}

/// define our functions for the GPT struct so we can use it later on
impl GPT {
    fn new() -> Self {
        GPT {
            disk_num: 0
        }
    }
}
