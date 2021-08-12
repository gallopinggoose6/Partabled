//! This defines our algorithm for determining what kinds of 
//! algorithmic shenanigans will be done to shift about partitions

// import the required MBR/GPT struct definitions
use crate::partitions::{
    GPT,
    MBR
};

// import the required helper functions
use crate::helpers::{
    get_free_ram_size,
};

