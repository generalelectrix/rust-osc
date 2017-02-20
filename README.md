*This repository is no longer maintained.  I'd recommend rosc instead, available on crates.io and at https://github.com/klingtnet/rosc .  It provides more extensive support for the full OSC spec and is pretty much identical in implementation as this package in all important respects.*

rust-osc
========

Library for sending and receiving OSC 1.0 streams over UDP.  At the moment I have
no plans for adding support for OSC 1.1 payloads, but there is already some skeleton
support for them commented out in this code.  This implementation leverages Rust's
ADT for representing all the different classes of Osc arguments and messages as
two enum types with variants.

This was compiled and tested under Rust 1.6 stable as of February 2016.

To compile the examples, compile src/osc-lib.rs into a library and tell the compiler
to link to it.
osc_sender_test.rs sends test data to the local host on port 7009, and
osc_receiver_test.rs listens to that port to receive.  Run the receiver in one
shell session and then run the sender in another to verify everything works.

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

(See doc/LICENSE.txt for a copy of the GPLv3.)
