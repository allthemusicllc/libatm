// midi_event.rs
//
// Copyright (c) 2020 All The Music, LLC
//
// This work is licensed under the Creative Commons Attribution 4.0 International License.
// To view a copy of this license, visit http://creativecommons.org/licenses/by/4.0/ or send
// a letter to Creative Commons, PO Box 1866, Mountain View, CA 94042, USA.

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
    /// let note = libatm::MIDINote::new(libatm::MIDINoteType::C, 4);
    /// let note_on_event = libatm::MIDIChannelVoiceMessage::new(0, &note, 0x64, libatm::MIDIStatus::NoteOn, 0);
    /// let note_off_event = libatm::MIDIChannelVoiceMessage::new(5, &note, 0, libatm::MIDIStatus::RunningStatus, 0);
    /// ```
    ///
    /// # Notes
    ///
    /// * The meaning of the delta_time unit is determined by the `division` value present
    ///   in the [MIDIHeader](struct.MIDIHeader.html).
    /// * A `NoteOn` event with a velocity of 0 is equivalent to a NoteOff event.  This library
    ///   heavily exploits this feature, as well as running status, to produce the smallest
    ///   possible MIDI files.
    /// * If the note type is [MIDINoteType::Empty](enum.MIDINoteType.html#variant.Empty)
    ///   then the velocity will automatically get set to 0.
    pub fn new(
        delta_time: u8,
        note: &crate::midi_note::MIDINote,
        velocity: u8,
        status: MIDIStatus,
        channel: u8,
    ) -> MIDIChannelVoiceMessage {
        // 0 <= channel < 0x10 (16)
        assert!(channel < 0x10);
        // 0 <= velocity < 0x80 (128)
        assert!(velocity < 0x80);

        // If note type is Empty, velocity must be 0
        let velocity = match note.note_type {
            crate::midi_note::MIDINoteType::Empty => 0u8,
            _ => velocity,
        };

        let event_status = match status {
            MIDIStatus::RunningStatus => 0,
            _ => (((status as u8) << 4) | channel),
        };

        MIDIChannelVoiceMessage {
            delta_time,
            status: event_status,
            note: (note.convert() as u8),
            velocity,
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
    /// let mut buffer = std::io::BufWriter::new(Vec::new());
    /// // Middle C
    /// let note = libatm::MIDINote::new(libatm::MIDINoteType::C, 4);
    /// // Play for 5 ticks
    /// let note_on_event = libatm::MIDIChannelVoiceMessage::new(0, &note, 0x64, libatm::MIDIStatus::NoteOn, 0);
    /// let note_off_event = libatm::MIDIChannelVoiceMessage::new(5, &note, 0, libatm::MIDIStatus::RunningStatus, 0);
    /// // Write notes to buffer
    /// note_on_event.write_buffer(&mut buffer).unwrap();
    /// note_off_event.write_buffer(&mut buffer).unwrap();
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
