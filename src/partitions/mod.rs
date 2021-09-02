
// re-export our modules
pub mod mbr;
pub mod gpt;

// export our commonly used structures
pub use mbr::MBR;
pub use gpt::{
    GPTDisk,
    GPTPartition
};