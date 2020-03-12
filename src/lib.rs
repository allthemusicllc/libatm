// lib.rs
//
// Copyright (c) 2020 All The Music, LLC
//
// This work is licensed under the Creative Commons Attribution 4.0 International License.
// To view a copy of this license, visit http://creativecommons.org/licenses/by/4.0/ or send
// a letter to Creative Commons, PO Box 1866, Mountain View, CA 94042, USA.

//! `libatm` is a library for generating and working with MIDI files.  It was purpose-built for
//! All the Music, LLC to assist in its mission to enable musicians to make all of their music
//! without the fear of frivolous copyright lawsuits.  All code is released into the public domain
//! via the [Creative Commons Attribution 4.0 International License](http://creativecommons.org/licenses/by/4.0/). 
//! If you're looking for a command line tool to generate and work with MIDI files, check out
//! [the `atm-cli` project](https://github.com/allthemusicllc/atm-cli) that utilizes this library.  For
//! more information on All the Music, check out [allthemusic.info](http://allthemusic.info).

// Allow dead code
#![allow(unused_parens)]

extern crate byteorder;
extern crate thiserror;

pub mod midi_event;
pub mod midi_file;
pub mod midi_note;

pub use midi_event::*;
pub use midi_file::*;
pub use midi_note::*;

// TODO: Finish writing tests for each module

// #[cfg(test)]
// mod tests {
//     use super::*;
// 
//     #[test]
//     fn test_midievent_noteon() {
//         let note = MIDINote::new(MIDINoteType::C, 4);
//         let event = MIDIChannelVoiceMessage::new(0, &note, 100, MIDIStatus::NoteOn, 0);
//         let status = 0b10010000;
//         assert_eq!(status, event.status);
//     }
// 
//     #[test]
//     fn test_midievent_runningstatus() {
//         let note = MIDINote::new(MIDINoteType::C, 4);
//         let event = MIDIChannelVoiceMessage::new(0, &note, 100, MIDIStatus::RunningStatus, 0);
//         assert_eq!(0, event.status);
//     }
// 
//     #[test]
//     fn test_midifile_size() {
//         let sequence = "C:4,D:4,E:4,C:4,D:4,E:4,C:4,D:4,E:4,C:4,D:4,E:4"
//             .parse::<MIDINoteSet>()
//             .unwrap();
//         let mfile = MIDIFile::new(sequence, MIDIFormat::Format0, 1, 1);
//         assert_eq!(95, mfile.gen_size());
//     }
// }
