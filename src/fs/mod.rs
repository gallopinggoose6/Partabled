use alloc::{string::String, vec::Vec};

/* 
NOTE THIS IS POTENTIALLY GONNA BE GOING UP INTO THE AIR
AND EXPLODING WITH VARYING DEGREES OF "WE DONT NEED THIS"
AT SOME UNDISCLOSED POINT IN THE FUTURE.  
*/
// re-export our file system modules
pub mod ext4;
pub mod fat32;


/// define raw file structure
pub struct RawFile{
    path: String,
    size: usize,
    data: Vec<u8>
}

/// define our Filesystem traits
pub trait Filesystem {
    /// creates a new instance of the filesystem
    fn new(media_id: u32, lba_start: u64, lba_end: u64) -> Self;

    fn get_file(&self, path: String) -> RawFile;
}