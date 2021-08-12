/* 
NOTE THIS IS POTENTIALLY GONNA BE GOING UP INTO THE AIR
AND EXPLODING WITH VARYING DEGREES OF "WE DONT NEED THIS"
AT SOME UNDISCLOSED POINT IN THE FUTURE.  
*/
// re-export our file system modules
pub mod ext4;
pub mod fat32;


/// define our Filesystem traits
pub trait Filesystem {
    /// creates a new instance of the filesystem
    fn new(media_id: u32, lba_start: u64, lba_end: u64) -> Self;

    /// defines the root of the filesystem
    fn root(&self);

    /// defines an iterator that gets all of the files in a directory
    fn file_iter(&self);
}