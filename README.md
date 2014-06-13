rust-osc
========

Library for sending and receiving OSC 1.0 streams over UDP.  At the moment I have
no plans for adding support for OSC 1.1 payloads, but there is already some skeleton
support for them commented out in this code.  This implementation leverages Rust's
ADT for representing all the different classes of Osc arguments and messages as
two enum types with variants.

This was compiled and tested under
rustc 0.11.0-pre-nightly (918dbfe 2014-06-02 20:51:30 -0700)

I will periodically update this to the latest nightly.  If you find bugs or if
compilation under the current nightly fails, please feel free to let me know!

To compile the examples, compile src/osc-lib.rs into a library and tell the compiler
to link to it.
osc_sender_test.rs sends test data to the local host on port 7009, and
osc_receiver_test.rs listens to that port to receive.  Run the receiver in one
shell session and then run the sender in another to verify everything works.

I've included broadcaster.rs in this library, which may be a useful tool for using OSC
streams in a multi-threaded concept.  The imagined use case is a task which manages
listening for OSC, and then sends the messages it receives to a list of consumers
running in different tasks.


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
