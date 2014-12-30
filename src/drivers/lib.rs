#![crate_name = "drivers"]
#![crate_type = "rlib"]
#![no_std]
#![feature(globs)]

extern crate core;
extern crate hil;

mod std {
    pub use core::*;
}

pub mod flash_attr;
pub mod timer;

