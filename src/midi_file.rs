// midi_file.rs
//
// Copyright (c) 2020 All The Music, LLC
//
// This work is licensed under the Creative Commons Attribution 4.0 International License.
// To view a copy of this license, visit http://creativecommons.org/licenses/by/4.0/ or send
// a letter to Creative Commons, PO Box 1866, Mountain View, CA 94042, USA.

use crate::midi_event::{MIDIChannelVoiceMessage, MIDIStatus};

/// MIDI file format
///
/// MIDI files have three different formats: 0, 1, and 2.  Format 0 means the MIDI file
/// has a single track chunk, whereas formats 1 and 2 indicate one _or more_ track chunks.
/// A longer discussion of these formats can be found in section 2.2 of the document here:
/// <https://www.cs.cmu.edu/~music/cmsip/readings/Standard-MIDI-file-format-updated.pdf>.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MIDIFormat {
    /// Single track.
    Format0,
    /// One or more simultaneous tracks.
    Format1,
    /// One or more independent tracks.
    Format2,
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
    pub fn new(chunk_type: Vec<u8>, length: u32) -> Self {
        Self { chunk_type, length }
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
    ) -> Self {
        Self {
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

/// Generate size of a MIDI track chunk in bytes given number of notes
pub fn gen_midi_track_size(num_notes: u32) -> u32 {
    (num_notes * 6) + 1
}

/// Generate the size of a MIDI file in bytes given number of notes
pub fn gen_midi_file_size(num_notes: u32) -> u32 {
    22 + gen_midi_track_size(num_notes)
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
    /// Sequence of notes to generate the track chunk from
    pub sequence: Vec<crate::midi_note::MIDINote>,
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
    /// # Examples
    ///
    /// ```rust
    /// let mfile = libatm::MIDIFile::new(
    ///     vec![
    ///         libatm::MIDINote::new(libatm::MIDINoteType::C, 4),
    ///         libatm::MIDINote::new(libatm::MIDINoteType::CSharp, 8),
    ///         libatm::MIDINote::new(libatm::MIDINoteType::D, 5),
    ///         libatm::MIDINote::new(libatm::MIDINoteType::DSharp, 3),
    ///     ],
    ///     libatm::MIDIFormat::Format0,
    ///     1,
    ///     1,
    /// );
    /// assert_eq!("601097451", mfile.gen_hash());
    /// ```
    pub fn new(
        sequence: Vec<crate::midi_note::MIDINote>,
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

    /// Generate unique hash for this file's `MIDINote` sequence
    ///
    /// This hash function simply concatenates the sequential integer
    /// representation of the file's sequence of `MIDINote`.  By this definition,
    /// no two non-identical sequences can have the same hash.  The primary
    /// intended purpose of this function is to allow for O(1) lookups by note sequence
    /// once a file has been written to disk.
    pub fn gen_hash(&self) -> String {
        self
            .sequence
            .iter()
            .map(|note| note.convert().to_string())
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

    /// Generate the size of this track chunk header in bytes (on disk)
    pub fn gen_track_size(&self) -> u32 {
        gen_midi_track_size(self.sequence.len() as u32)
    }

    /// Generate track chunk header (see: [MIDITrackHeader](struct.MIDITrackHeader.html))
    pub fn gen_track_header(&self) -> MIDITrackHeader {
        MIDITrackHeader::new(
            vec![0x4d, 0x54, 0x72, 0x6b], // 'MTrk'
            self.gen_track_size(),
        )
    }

    /// Generate track data (see: [MIDIChannelVoiceMessage](../midi_event/struct.MIDIChannelVoiceMessage.html))
    pub fn gen_track(&self) -> Vec<MIDIChannelVoiceMessage> {
        let delta_time = self.division as u8;
        self
            .sequence
            .iter()
            .enumerate()
            .map(|(idx, note)| {
                let first_status = match idx {
                    0 => MIDIStatus::NoteOn,
                    _ => MIDIStatus::RunningStatus,
                };
                vec![
                    MIDIChannelVoiceMessage::new(0, &note, 0x64, first_status, 0,),
                    MIDIChannelVoiceMessage::new(delta_time, &note, 0, MIDIStatus::RunningStatus, 0,)
                ]
            })
            .flatten()
            .collect::<Vec<MIDIChannelVoiceMessage>>()
    }

    /// Generate the size of this MIDI file in bytes (on disk)
    pub fn gen_size(&self) -> u32 {
        gen_midi_file_size(self.sequence.len() as u32)
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
    pub fn gen_file(&self) -> std::io::Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(self.gen_size() as usize);
        self.write_buffer(&mut buffer)?;
        Ok(buffer)
    }

    /// Write MIDI file to path on disk
    pub fn write_file<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        let target_file = std::fs::File::create(path)?;
        let mut target_file = std::io::BufWriter::new(target_file);
        self.write_buffer(&mut target_file)?;
        Ok(())
    }
}
