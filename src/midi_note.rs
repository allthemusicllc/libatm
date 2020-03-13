// midi_note.rs
//
// Copyright (c) 2020 All The Music, LLC
//
// This work is licensed under the Creative Commons Attribution 4.0 International License.
// To view a copy of this license, visit http://creativecommons.org/licenses/by/4.0/ or send
// a letter to Creative Commons, PO Box 1866, Mountain View, CA 94042, USA.

/// Error type for parsing [MIDINoteType](struct.MIDINoteType.html) from `&str`
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ParseMIDINoteTypeError {
    #[error("Unknown note type {input}")]
    UnknownNoteType { input: String },
}

/// MIDI note type
///
/// Represents each note in an octave, where each "*Sharp" value
/// is an enharmonic key.  Each note type must be combined with an
/// integer value to fully represent a key on the piano (see: [MIDINote](struct.MIDINote.html)).
/// The `Rest` note type represents silence, or the absence of a note.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
    Rest,
}

impl<'a> std::str::FromStr for MIDINoteType {
    type Err = ParseMIDINoteTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bsharp"
            | "b#"
            | "c" => Ok(Self::C),
            "csharp"
            | "c#"
            | "dflat"
            | "d♭"
            | "db" => Ok(Self::CSharp),
            "d" => Ok(Self::D),
            "dsharp"
            | "d#"
            | "eflat"
            | "e♭"
            | "eb" => Ok(Self::DSharp),
            "e"
            | "fflat"
            | "f♭"
            | "fb" => Ok(Self::E),
            "esharp"
            | "e#"
            | "f" => Ok(Self::F),
            "fsharp"
            | "f#"
            | "gflat"
            | "g♭"
            | "gb" => Ok(Self::FSharp),
            "g" => Ok(Self::G),
            "gsharp"
            | "g#"
            | "aflat"
            | "a♭"
            | "ab" => Ok(Self::GSharp),
            "a" => Ok(Self::A),
            "asharp"
            | "a#"
            | "bflat"
            | "b♭"
            | "bb" => Ok(Self::ASharp),
            "cflat"
            | "c♭"
            | "cb"
            | "b" => Ok(Self::B),
            "rest" | "empty" => Ok(Self::Rest),
            _ => Err(ParseMIDINoteTypeError::UnknownNoteType {
                input: s.to_string(),
            }),
        }
    }
}

/// Error type for parsing [MIDINote](struct.MIDINote.html) from `&str`
///
/// Credit to [@ldesgoui](https://github.com/ldesgoui) for the suggestion.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ParseMIDINoteError {
    #[error("Invalid note format (expected '<note>:<octave>', found {input})")]
    InvalidNoteFormat { input: String },
    #[error(transparent)]
    InvalidOctave(#[from] std::num::ParseIntError),
    #[error(transparent)]
    UnknownNoteType(#[from] ParseMIDINoteTypeError),
}

/// MIDI note
///
/// Represents key on a piano, combining a [note type](enum.MIDINoteType.html)
/// with an octave.  For example, middle C would be represented as
/// `MIDINote { note_type: MIDINoteType::C, octave: 4 }`.  For a detailed table
/// of MIDI notes and octave numbers, see document here:
/// <https://www.cs.cmu.edu/~music/cmsip/readings/Standard-MIDI-file-format-updated.pdf>.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
    /// let note = libatm::MIDINote::new(libatm::MIDINoteType::C, 4);
    /// assert_eq!(60, note.convert());
    /// ```
    ///
    /// # Notes
    ///
    /// The `octave` parameter is not validated, but must be between
    /// -1 and 9 in order to represent a valid MIDI note.
    pub fn new(note_type: MIDINoteType, octave: u32) -> Self {
        Self { note_type, octave, }
    }

    /// Convert MIDI note to an integer representation (MIDI note number)
    ///
    /// The empty note (A.K.A. silence) is represented as `u32::max_value()`.
    /// The MIDI Tuning Standard only represents pitches up to 127, but instead
    /// of choosing some arbitrary number larger than 127, `u32::max_value()` is easily
    /// distinguishable.
    pub fn convert(&self) -> u32 {
        match &self.note_type {
            MIDINoteType::Rest => u32::max_value(),
            _ => (self.note_type as u32) + (self.octave + 1) * 12,
        }
    }
}

impl<'a> std::str::FromStr for MIDINote {
    type Err = ParseMIDINoteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split input on ':' to get note type and octave
        let split_pair: Vec<&str> = s.split(':').collect();
        // Ensure is a pair (length is 2)
        if split_pair.len() != 2 {
            return Err(ParseMIDINoteError::InvalidNoteFormat {
                input: s.to_string(),
            });
        }
        // Parse MIDINoteType from first item in pair
        let note_type = MIDINoteType::from_str(split_pair[0])?;
        // Parse octave (as u32) from second item in pair
        // TODO: Enforce octave range of -1 to 9
        let octave = split_pair[1].parse::<u32>()?;
        Ok(Self { note_type, octave })
    }
}

/// Error type for parsing [MIDINoteSet](struct.MIDINoteSet.html) from `&str`
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ParseMIDINoteSetError {
    #[error("Invalid note at index {0}")]
    ParseMIDINote(usize, #[source] ParseMIDINoteError),
}

/// Container for set of `MIDINote`
///
/// Implements the [FromStr](https://doc.rust-lang.org/nightly/core/str/trait.FromStr.html)
/// trait as a convenience method for parsing a set of `MIDINote` (from a command line
/// argument).
///
/// # Examples
///
/// ```rust
/// // Parse MIDI note set from &str
/// let note_set = "C:4,D:5,CSharp:8,DSharp:3".parse::<libatm::MIDINoteSet>().unwrap();
/// // Expected note set contains C:4, D:5, CSharp:8, and DSharp:3
/// let expected = libatm::MIDINoteSet::new(vec![
///     libatm::MIDINote::new(libatm::MIDINoteType::C, 4),
///     libatm::MIDINote::new(libatm::MIDINoteType::D, 5),
///     libatm::MIDINote::new(libatm::MIDINoteType::CSharp, 8),
///     libatm::MIDINote::new(libatm::MIDINoteType::DSharp, 3),
/// ].into_iter().collect::<std::collections::BTreeSet<libatm::MIDINote>>());
/// assert_eq!(expected, note_set);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MIDINoteSet(std::collections::BTreeSet<MIDINote>);

impl MIDINoteSet {
    pub fn new(notes: std::collections::BTreeSet<MIDINote>) -> Self {
        Self(notes)
    }
}

impl std::str::FromStr for MIDINoteSet {
    type Err = ParseMIDINoteSetError;

    /// Credit to [@ldesgoui](https://github.com/ldesgoui) for the implementation
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let notes = s
            .split(',')
            .enumerate()
            .map(|(idx, pair)| {
                pair.parse::<MIDINote>()
                    .map_err(|err| ParseMIDINoteSetError::ParseMIDINote(idx, err))
            })
            .collect::<Result<std::collections::BTreeSet<MIDINote>, ParseMIDINoteSetError>>()?;
        Ok(Self(notes))
    }
}

impl std::ops::Deref for MIDINoteSet {
    type Target = std::collections::BTreeSet<MIDINote>;

    /// Allow dereferencing of tuple struct to underlying hash set
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_note_from_str_valid() {
        let observed = "C#:5".parse::<MIDINote>();
        let expected = Ok(MIDINote::new(MIDINoteType::CSharp, 5));
        assert_eq!(expected, observed);
    }

    #[test]
    fn test_midi_note_from_str_invalid_note_type() {
        let observed = "C$:5".parse::<MIDINote>();
        let expected = Err(
            ParseMIDINoteError::UnknownNoteType(
                ParseMIDINoteTypeError::UnknownNoteType { input: "C$".to_string() }
            )
        );
        assert_eq!(expected, observed);
    }

    #[test]
    fn test_midi_note_from_str_invalid_format() {
        let input = "C#;5".to_string();
        let observed = input.as_str().parse::<MIDINote>();
        let expected = Err(ParseMIDINoteError::InvalidNoteFormat { input });
        assert_eq!(expected, observed);
    }

    #[test]
    fn test_midi_note_from_str_invalid_octive() {
        let input = "C#:12.3".to_string();
        let observed = input.as_str().parse::<MIDINote>();
        // NOTE: Cannot compare to expected here because pattern matching on
        //       std::num::IntErrorKind is considered unstable.
        assert!(observed.is_err());
    }

    #[test]
    fn test_midi_note_set_from_str_valid_no_duplicate() {
        let observed = "C:4,D:4,E:4,F:4,F#:4,DFlat:5".parse::<MIDINoteSet>();
        let expected = Ok(MIDINoteSet::new(vec![
            MIDINote::new(MIDINoteType::C, 4),
            MIDINote::new(MIDINoteType::D, 4),
            MIDINote::new(MIDINoteType::E, 4),
            MIDINote::new(MIDINoteType::F, 4),
            MIDINote::new(MIDINoteType::FSharp, 4),
            MIDINote::new(MIDINoteType::CSharp, 5),
        ].into_iter().collect::<std::collections::BTreeSet<MIDINote>>()));
        assert_eq!(expected, observed);
    }

    #[test]
    fn test_midi_note_set_from_str_valid_with_duplicate() {
        let observed = "C:4,C:4,D:5".parse::<MIDINoteSet>();
        let expected = Ok(MIDINoteSet::new(vec![
            MIDINote::new(MIDINoteType::C, 4),
            MIDINote::new(MIDINoteType::D, 5),
        ].into_iter().collect::<std::collections::BTreeSet<MIDINote>>()));
        assert_eq!(expected, observed);
    }

    #[test]
    fn test_midi_note_set_from_str_invalid_note_type() {
        // NOTE: Any invalid MIDINote in the input sequence
        //       should trigger a ParseMIDINoteError, so only
        //       one is tested here (picked arbitarily).
        let input = "C:4,CL:8,D:6".to_string();
        let observed = input.as_str().parse::<MIDINoteSet>();
        let expected = Err(ParseMIDINoteSetError::ParseMIDINote(
            1, 
            ParseMIDINoteError::UnknownNoteType(ParseMIDINoteTypeError::UnknownNoteType { input: "CL".to_string(), }),
        ));
        assert_eq!(expected, observed);
    }

    #[test]
    fn test_midi_note_set_from_str_extra_comma() {
        // NOTE: Any invalid MIDINote in the input sequence
        //       should trigger a ParseMIDINoteError, so only
        //       one is tested here (picked arbitarily).
        let input = "C:4,C:8,D:6,".to_string();
        let observed = input.as_str().parse::<MIDINoteSet>();
        let expected = Err(ParseMIDINoteSetError::ParseMIDINote(
            3, 
            ParseMIDINoteError::InvalidNoteFormat { input: "".to_string() },
        ));
        assert_eq!(expected, observed);
    }
}
