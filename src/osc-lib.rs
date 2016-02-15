#![crate_name = "osc"]

// if you have made changes and are re-compiling, you may want to turn these warnings back on
#![allow(unused_must_use)]
#![allow(unused_imports)]

#[macro_use]
mod osc_util;  // must declare mods with macro exports in them before users!
pub mod osc_receiver;
pub mod osc_sender;
#[macro_use]
pub mod osc_data;
