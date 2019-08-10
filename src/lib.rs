// lib.rs
//
// Copyright (c) 2019 All The Music, LLC
//
// This work is licensed under the Creative Commons Attribution 4.0 International License.
// To view a copy of this license, visit http://creativecommons.org/licenses/by/4.0/ or send
// a letter to Creative Commons, PO Box 1866, Mountain View, CA 94042, USA.

//! `libatm` is a library for generating and working with MIDI files.  It was purpose-built for
//! All the Music, LLC to assist in its mission to enable musicians to make all of their music
//! without the fear of frivolous copyright lawsuits.  All code is released into the public domain
//! via the [Creative Commons Attribute 4.0 International License](http://creativecommons.org/licenses/by/4.0/). 
//! If you're looking for a command line tool to generate and work with MIDI files, check out
//! [the `atm-cli` project](https://github.com/allthemusicllc/atm-cli) that utilizes this library.  For
//! more information on All the Music, check out [allthemusic.info](http://allthemusic.info).

// Allow dead code
#![allow(unused_parens)]

extern crate byteorder;

/// MIDI file format
///
/// MIDI files have three different formats: 0, 1, and 2.  Format 0 means the MIDI file
/// has a single track chunk, whereas formats 1 and 2 indicate one _or more_ track chunks.
/// A longer discussion of these formats can be found in section 2.2 of the document here:
/// <https://www.cs.cmu.edu/~music/cmsip/readings/Standard-MIDI-file-format-updated.pdf>.
#[derive(Clone, Copy, Debug)]
pub enum MIDIFormat {
    /// Single track.
    Format0,
    /// One or more simultaneous tracks.
    Format1,
    /// One or more independent tracks.
    Format2,
}

/// MIDI message status
///
/// Each MIDI event (message) has a status, which sets the message type and thus the meaning
/// of the associated message data.  Technically the status bits also include the channel number,
/// but this library currently only supports single track, single channel MIDI files (and thus
/// defaults to channel 0).  For a detailed description of each status type, see Appendix 1.1 of the document here:
/// <https://www.cs.cmu.edu/~music/cmsip/readings/Standard-MIDI-file-format-updated.pdf>.
#[derive(Clone, Copy, Debug)]
pub enum MIDIStatus {
    /// Assume status bytes of previous MIDI channel message
    RunningStatus = 0b0000,
    /// Note released
    NoteOff = 0b1000,
    /// Note pressed
    NoteOn = 0b1001,
    /// Pressure on key after pressed down
    PolyphonicAftertouch = 0b1010,
    /// Controller value change
    ControlChange = 0b1011,
    /// Change program (patch) number
    ProgramChange = 0b1100,
    /// Greatest pressure on key after pressed down
    Aftertouch = 0b1101,
    /// Chainge pitch wheel
    PitchWheelChange = 0b1110,
}

/// MIDI note type
///
/// Represents each note in an octave, where each "*Sharp" value
/// is an enharmonic key.  Each note type must be combined with an
/// integer value to fully represent a key on the piano (see: [MIDINote](struct.MIDINote.html)).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MIDINoteType {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl std::str::FromStr for MIDINoteType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C" => Ok(MIDINoteType::C),
            "CSharp" => Ok(MIDINoteType::CSharp),
            "D" => Ok(MIDINoteType::D),
            "DSharp" => Ok(MIDINoteType::DSharp),
            "E" => Ok(MIDINoteType::E),
            "F" => Ok(MIDINoteType::F),
            "FSharp" => Ok(MIDINoteType::FSharp),
            "G" => Ok(MIDINoteType::G),
            "GSharp" => Ok(MIDINoteType::GSharp),
            "A" => Ok(MIDINoteType::A),
            "ASharp" => Ok(MIDINoteType::ASharp),
            "B" => Ok(MIDINoteType::B),
            _ => Err(()),
        }
    }
}

/// MIDI note
///
/// Represents key on a piano, combining a [note type](enum.MIDINoteType.html)
/// with an octave.  For example, middle C would be represented as
/// `MIDINote { note_type: MIDINoteType::C, octave: 4 }`.  For a detailed table
/// of MIDI notes and octave numbers, see document here:
/// <https://www.cs.cmu.edu/~music/cmsip/readings/Standard-MIDI-file-format-updated.pdf>.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MIDINote {
    pub note_type: MIDINoteType,
    pub octave: u32,
}

impl MIDINote {
    /// Create new `MIDINote`
    ///
    /// # Arguments
    ///
    /// * `note_type`: [MIDINoteType](enum.MIDINoteType.html)
    /// * `octave`: integer between -1 and 9
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Middle C
    /// let note = MIDINote::new(MIDINoteType::C, 4);
    /// assert_eq!(note.convert(), 60);
    /// ```
    ///
    /// # Notes
    ///
    /// The `octave` parameter is not validated, but must be between
    /// -1 and 9 in order to represent a valid MIDI note.
    pub fn new(note_type: MIDINoteType, octave: u32) -> MIDINote {
        MIDINote { note_type, octave }
    }

    /// Convert MIDI note to an integer representation (MIDI note number)
    pub fn convert(&self) -> u32 {
        return (self.note_type as u32) + ((self.octave + 1) * 12);
    }
}

impl std::str::FromStr for MIDINote {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_pair: Vec<&str> = s.split(':').collect();
        let note_type = split_pair[0].parse::<MIDINoteType>()?;
        let octave = split_pair[1].parse::<u32>().map_err(|_| ())?;
        Ok(MIDINote { note_type, octave })
    }
}

/// Container for sequence of `MIDINote`
///
/// Implements the [FromStr](https://doc.rust-lang.org/nightly/core/str/trait.FromStr.html)
/// trait as a convenience method for parsing a `MIDINote` sequence from a command line
/// argument.
///
/// # Examples
///
/// ```rust
/// let sequence = "C:4,D:5,CSharp:8,DSharp:3".parse::<MIDINoteSequence>().unwrap();
/// assert_eq!(sequence, MIDINoteSequence::new(vec![
///     MIDINote::new(MIDINote::C, 4),
///     MIDINote::new(MIDINote::D, 5),
///     MIDINote::new(MIDINote::CSharp, 8),
///     MIDINote::new(MIDINote::DSharp, 3),
/// ]));
/// ```
#[derive(Clone, Debug)]
pub struct MIDINoteSequence {
    pub notes: Vec<MIDINote>,
}

impl MIDINoteSequence {
    pub fn new(notes: Vec<MIDINote>) -> MIDINoteSequence {
        MIDINoteSequence { notes }
    }
}

impl std::str::FromStr for MIDINoteSequence {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let notes: Vec<&str> = s.split(",").collect();
        let notes: Vec<MIDINote> = notes
            .iter()
            .map(|&pair| pair.parse::<MIDINote>().unwrap())
            .collect::<Vec<MIDINote>>();
        Ok(MIDINoteSequence { notes })
    }
}

/// MIDI channel voice message
///
/// MIDI supports two main types of messages: Channel and System.
/// Channel messages are tied to a specific MIDI channel, whereas
/// System messages are not (and thus don't contain a channel number).
/// This library only supports channel messages, and more specifically
/// the `NoteOn` and `NoteOff` channel _voice_ messages,
/// which actually produce sounds.  For a detailed explanation of
/// MIDI messages, see appendix 1.1 of the document here:
/// <https://www.cs.cmu.edu/~music/cmsip/readings/Standard-MIDI-file-format-updated.pdf>.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MIDIChannelVoiceMessage {
    pub delta_time: u8,
    pub status: u8,
    pub note: u8,
    pub velocity: u8,
}

impl MIDIChannelVoiceMessage {
    /// Create new `MIDIChannelVoiceMessage`
    ///
    /// # Arguments
    ///
    /// * `delta_time`: time delta since last MIDI channel message
    /// * `note`: [MIDINote](struct.MIDINote.html) to play
    /// * `velocity`: velocity with which to play the note
    /// * `status`: [MIDIStatus](enum.MIDIStatus.html) bits of the message
    /// * `channel`: channel on which to play the message
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Create Middle C note and two MIDI events, one to "press" the key and
    /// // one to "release" they key after 5 ticks.
    /// let note = MIDINote(MIDINoteType::C, 4);
    /// let note_on_event = MIDIChannelVoiceMessage::new(0, &note, 0x64, MIDIStatus::NoteOn, 0);
    /// let note_off_event = MIDIChannelVoiceMessage::new(5, &note, 0, MIDIStatus::RunningStatus, 0);
    /// ```
    ///
    /// # Notes
    ///
    /// * The meaning of the delta_time unit is determined by the `division` value present 
    ///   in the [MIDIHeader](struct.MIDIHeader.html).
    /// * A `NoteOn` event with a velocity of 0 is equivalent to a NoteOff event.  This library
    ///   heavily exploits this feature, as well as running status, to produce the smallest
    ///   possible MIDI files.
    pub fn new(
        delta_time: u8,
        note: &MIDINote,
        velocity: u8,
        status: MIDIStatus,
        channel: u8,
    ) -> MIDIChannelVoiceMessage {
        assert!(channel < 0x10); // 0 <= channel < 0x10 (16)
        assert!(velocity < 0x80); // 0 <= velocity < 0x80 (128)
        let event_status = match status {
            MIDIStatus::RunningStatus => 0,
            _ => (((status as u8) << 4) | channel),
        };
        MIDIChannelVoiceMessage {
            delta_time: delta_time,
            status: event_status,
            note: (note.convert() as u8),
            velocity: velocity,
        }
    }

    /// Write MIDI channel message to buffer
    ///
    /// # Arguments
    ///
    /// * `target`: buffer to write to
    ///
    /// # Examples
    ///
    /// ```rust
    /// use byteorder::WriteBytesExt;
    ///
    /// // Target buffer
    /// let buffer = std::io::BufWriter::new(Vec::new());
    /// // Middle C
    /// let note = MIDINote(MIDINoteType::C, 4);
    /// // Play for 5 ticks
    /// let note_on_event = MIDIChannelVoiceMessage::new(0, &note, 0x64, MIDIStatus::NoteOn, 0);
    /// let note_off_event = MIDIChannelVoiceMessage::new(5, &note, 0, MIDIStatus::RunningStatus, 0);
    /// // Write notes to buffer
    /// note_on_event.write_buffer(buffer).unwrap();
    /// note_off_event.write_buffer(buffer).unwrap();
    /// ```
    pub fn write_buffer<T>(&self, target: &mut T) -> std::io::Result<()>
    where
        T: byteorder::WriteBytesExt,
    {
        target.write_u8(self.delta_time)?;
        if self.status != 0 {
            target.write_u8(self.status)?;
        }
        target.write_u8(self.note)?;
        target.write_u8(self.velocity)?;
        Ok(())
    }
}

/// MIDI track chunk header
///
///  Encapsulates the chunk type ('MTrk') and the length
///  of a MIDI track chunk.  The official MIDI spec does
///  not refer to these data as the truck chunk header, this
///  library simply makes the distinction for ease of use.
#[derive(Clone, Debug, PartialEq)]
pub struct MIDITrackHeader {
    pub chunk_type: Vec<u8>,
    pub length: u32,
}

impl MIDITrackHeader {
    /// Create new `MIDITrackHeader`
    pub fn new(chunk_type: Vec<u8>, length: u32) -> MIDITrackHeader {
        MIDITrackHeader { chunk_type, length }
    }

    /// Write track chunk header to buffer
    pub fn write_buffer<T>(&self, target: &mut T) -> std::io::Result<()>
    where
        T: byteorder::WriteBytesExt,
    {
        for elem in self.chunk_type.iter() {
            target.write_u8(*elem)?;
        }
        target.write_u32::<byteorder::BigEndian>(self.length)?;
        Ok(())
    }
}

/// MIDI file header
///
/// Unlike the [MIDITrackHeader](struct.MIDITrackHeader.html), this structure is
/// specified in the official MIDI spec (as "Header Chunk"), though the last three 16-bit
/// fields are simply referred to as "Data".  For a more detailed discussion of the
/// Header Chunk, see section 2.1 of the document here:
/// <https://www.cs.cmu.edu/~music/cmsip/readings/Standard-MIDI-file-format-updated.pdf>.
#[derive(Clone, Debug, PartialEq)]
pub struct MIDIHeader {
    pub chunk_type: Vec<u8>,
    pub length: u32,
    pub format: u16,
    pub tracks: u16,
    pub division: u16,
}

impl MIDIHeader {
    /// Create new `MIDIHeader`
    pub fn new(
        chunk_type: Vec<u8>,
        length: u32,
        format: MIDIFormat,
        tracks: u16,
        division: u16,
    ) -> MIDIHeader {
        MIDIHeader {
            chunk_type,
            length,
            format: format as u16,
            tracks,
            division,
        }
    }

    /// Write header chunk to buffer
    pub fn write_buffer<T>(&self, target: &mut T) -> std::io::Result<()>
    where
        T: byteorder::WriteBytesExt,
    {
        for elem in self.chunk_type.iter() {
            target.write_u8(*elem)?;
        }
        target.write_u32::<byteorder::BigEndian>(self.length)?;
        target.write_u16::<byteorder::BigEndian>(self.format)?;
        target.write_u16::<byteorder::BigEndian>(self.tracks)?;
        target.write_u16::<byteorder::BigEndian>(self.division)?;
        Ok(())
    }
}

/// MIDI file representation
///
/// MIDI files can be complex, allowing for any number of tracks with
/// different notes and instruments playing simultaneously.  This library
/// was created for the express purpose of brute-forcing melodies, and thus
/// only supports a subset of the official MIDI standard.  More specifically,
/// this class is optimized for creating the smallest possible single track MIDI
/// files.
#[derive(Clone, Debug)]
pub struct MIDIFile {
    /// Sequence of notes ([MIDINoteSequence](struct.MIDINoteSequence.html)) from which the track chunk is generated
    pub sequence: MIDINoteSequence,
    /// Format specification (should always be [MIDIFormat::0](enum.MIDIFormat.html#variant.Format0))
    pub format: MIDIFormat,
    /// Number of tracks in MIDI file (should always be `1`)
    pub tracks: u16,
    /// Number of ticks to represent a quarter-note (recommended to use `1`)
    pub division: u16,
}

impl MIDIFile {
    /// Create new `MIDIFile`
    ///
    /// # Arguments
    ///
    /// See field comments [above](struct.MIDIFile.html#structfield.sequence).
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mfile = MIDIFile(
    ///     MIDINoteSequence::new(vec![
    ///         MIDINote::new(MIDINote::C, 4),
    ///         MIDINote::new(MIDINote::D, 5),
    ///         MIDINote::new(MIDINote::CSharp, 8),
    ///         MIDINote::new(MIDINote::DSharp, 3),
    ///     ]),
    ///     MIDIFormat::0,
    ///     1,
    ///     1,
    /// );
    /// assert_eq!("607410951", mfile.gen_hash());
    /// ```
    pub fn new(
        sequence: MIDINoteSequence,
        format: MIDIFormat,
        tracks: u16,
        division: u16,
    ) -> MIDIFile {
        MIDIFile {
            sequence,
            format,
            tracks,
            division,
        }
    }

    /// Generate unique hash for this file's `MIDINoteSequence`
    ///
    /// This hash function simply concatenates the sequential integer
    /// representation of the file's `MIDINotesequence`.  By this definition,
    /// no two non-identical sequences can have the same hash.  The primary
    /// intended purpose of this function is to allow for O(1) lookups by note sequence
    /// once a file has been written to disk, and thus there is no requirement
    /// to mitigate collisions for identical sequences.
    pub fn gen_hash(&self) -> String {
        self.sequence
            .notes
            .iter()
            .map(|&note| note.convert().to_string())
            .collect::<Vec<String>>()
            .join("")
    }

    /// Generate header chunk (see: [MIDIHeader](struct.MIDIHeader.html))
    pub fn gen_header(&self) -> MIDIHeader {
        MIDIHeader::new(
            vec![0x4d, 0x54, 0x68, 0x64], // 'MThd'
            0x06,
            self.format,
            self.tracks,
            self.division,
        )
    }

    /// Generate the size in bytes of the track chunk once written to disk
    pub fn gen_track_size(&self) -> u32 {
        (((self.sequence.notes.len() as u32) * 6) + 1)
    }

    /// Generate track chunk header (see: [MIDITrackHeader](struct.MIDITrackHeader.html))
    pub fn gen_track_header(&self) -> MIDITrackHeader {
        MIDITrackHeader::new(
            vec![0x4d, 0x54, 0x72, 0x6b], // 'MTrk'
            self.gen_track_size(),
        )
    }

    /// Generate track data (see: [MIDIChannelVoiceMessage](struct.MIDIChannelVoiceMessage.html))
    pub fn gen_track(&self) -> Vec<MIDIChannelVoiceMessage> {
        let mut track: Vec<MIDIChannelVoiceMessage> = Vec::with_capacity(self.sequence.notes.len() * 2);
        let delta_time = self.division as u8;
        for (idx, &note) in self.sequence.notes.iter().enumerate() {
            let note_on_event = match idx {
                0 => MIDIChannelVoiceMessage::new(0, &note, 0x64, MIDIStatus::NoteOn, 0),
                _ => MIDIChannelVoiceMessage::new(0, &note, 0x64, MIDIStatus::RunningStatus, 0),
            };
            let note_off_event = MIDIChannelVoiceMessage::new(delta_time, &note, 0, MIDIStatus::RunningStatus, 0);
            track.push(note_on_event);
            track.push(note_off_event);
        }
        track
    }

    /// Generate the size in bytes of this MIDI file once written to disk
    pub fn gen_size(&self) -> u32 {
        22 + self.gen_track_size()
    }

    /// Write MIDI file to buffer
    pub fn write_buffer<T>(&self, target: &mut T) -> std::io::Result<()>
    where
        T: byteorder::WriteBytesExt,
    {
        let header = self.gen_header();
        header.write_buffer(target)?;

        let track_header = self.gen_track_header();
        track_header.write_buffer(target)?;

        let track = self.gen_track();
        for event in track.iter() {
            event.write_buffer(target)?;
        }
        Ok(())
    }

    /// Generate buffer containing entire MIDI file
    pub fn gen_buffer(&self) -> std::io::Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(self.gen_size() as usize);   
        self.write_buffer(&mut buffer)?;
        Ok(buffer)
    }

    /// Write MIDI file to path on disk
    pub fn write_file(&self, path: &str) -> std::io::Result<()> {
        let target_file = std::fs::File::create(path)?;
        let mut target_file = std::io::BufWriter::new(target_file);
        self.write_buffer(&mut target_file)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midinote_fromstr() {
        let note = "C:4".parse::<MIDINote>().unwrap();
        assert!(note.note_type == MIDINoteType::C && note.octave == 4);
    }

    #[test]
    fn test_midinote_convert() {
        let note = "C:4".parse::<MIDINote>().unwrap();
        assert_eq!(60, note.convert());
    }

    #[test]
    fn test_midinotesequence_fromstr() {
        let sequence = "C:4,D:4,E:4".parse::<MIDINoteSequence>().unwrap();
        assert!(
            sequence.notes.len() == 3
                && sequence.notes[0] == MIDINote::new(MIDINoteType::C, 4)
                && sequence.notes[1] == MIDINote::new(MIDINoteType::D, 4)
                && sequence.notes[2] == MIDINote::new(MIDINoteType::E, 4)
        );
    }

    #[test]
    fn test_midievent_noteon() {
        let note = MIDINote::new(MIDINoteType::C, 4);
        let event = MIDIChannelVoiceMessage::new(0, &note, 100, MIDIStatus::NoteOn, 0);
        let status = 0b10010000;
        assert_eq!(status, event.status);
    }

    #[test]
    fn test_midievent_runningstatus() {
        let note = MIDINote::new(MIDINoteType::C, 4);
        let event = MIDIChannelVoiceMessage::new(0, &note, 100, MIDIStatus::RunningStatus, 0);
        assert_eq!(0, event.status);
    }

    #[test]
    fn test_midifile_size() {
        let sequence = "C:4,D:4,E:4,C:4,D:4,E:4,C:4,D:4,E:4,C:4,D:4,E:4"
            .parse::<MIDINoteSequence>()
            .unwrap();
        let mfile = MIDIFile::new(sequence, MIDIFormat::Format0, 1, 1);
        assert_eq!(95, mfile.gen_size());
    }
}
