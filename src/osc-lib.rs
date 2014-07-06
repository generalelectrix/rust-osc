#![crate_name = "osc"]

#![feature(globs)]
#![feature(struct_variant)]
#![feature(macro_rules)]
// if you have made changes and are re-compiling, you may want to turn these warnings back on
#![allow(unused_must_use)]
#![allow(dead_code)]
#![allow(unused_imports)]

mod osc_util;  // must declare mods with macro exports in them before users!
pub mod osc_receiver;
pub mod osc_sender;
pub mod osc_data;
