#![crate_name = "osc"]

#![feature(duration)]
#![feature(socket_timeout)]
#![feature(convert)]
#![feature(str_char)]
#![feature(io)]
// if you have made changes and are re-compiling, you may want to turn these warnings back on
#![allow(unused_must_use)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
mod osc_util;  // must declare mods with macro exports in them before users!
pub mod osc_receiver;
pub mod osc_sender;
#[macro_use]
pub mod osc_data;
