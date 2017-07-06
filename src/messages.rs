use formats::*;

pub trait MessageTool {
    fn binary(&self) -> Vec<u8>;
    fn len(&self) -> usize;
    fn status_byte(&self) -> u8;
}

#[derive(PartialEq, Clone, Debug)]
pub enum Message {
    MetaEvent {
        delta_time: VLQ,
        event: MetaEvent,
        data: Vec<u8>,
    },
    MidiEvent { delta_time: VLQ, event: MidiEvent },
    SysExEvent {
        delta_time: VLQ,
        event: SysExEvent,
        data: Vec<u8>,
    },
    TrackChange,
}
impl Message {
    pub fn binary(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();
        use messages::Message::*;
        match *self {
            MetaEvent {
                delta_time,
                ref event,
                ref data,
            } => {
                binary.append(&mut delta_time.binary());
                binary.append(&mut event.binary());
                binary.extend_from_slice(&VLQ::new(data.len() as u32).binary());
                binary.extend_from_slice(&data);
            }
            MidiEvent {
                delta_time,
                ref event,
            } => {
                binary.append(&mut delta_time.binary());
                binary.append(&mut event.binary());
            }
            SysExEvent {
                delta_time,
                ref event,
                ref data,
            } => {
                binary.append(&mut delta_time.binary());
                binary.append(&mut event.binary());
                use messages::SysExEvent::*;
                match *event {
                    F0 => {
                        binary.append(&mut VLQ::new(data.len() as u32 - 1).binary());
                        binary.extend_from_slice(&data[1..]);
                    }
                    _ => {
                        binary.append(&mut VLQ::new(data.len() as u32).binary());
                        binary.extend_from_slice(&data);
                    }
                }
            }
            TrackChange => binary.append(&mut Tag::Track.binary().to_vec()),
        }
        binary
    }
    pub fn len(&self) -> usize {
        use messages::Message::*;
        match *self {
            MetaEvent {
                delta_time,
                ref event,
                ref data,
            } => delta_time.len() + event.len() + (VLQ::new(data.len() as u32).len()) + data.len(),
            MidiEvent {
                delta_time,
                ref event,
            } => delta_time.len() + event.len(),
            SysExEvent {
                delta_time,
                ref event,
                ref data,
            } => {
                use messages::SysExEvent::*;
                delta_time.len() +
                    VLQ::new(
                        data.len() as u32 +
                            match *event {
                                F0 => 1,
                                _ => 0,
                            },
                    ).len() + data.len()
            }
            TrackChange => Tag::Track.binary().len(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum MetaEvent {
    SequenceNumber,
    TextEvent,
    CopyrightNotice,
    SequenceOrTrackName,
    InstrumentName,
    Lyric,
    Marker,
    CuePoint,
    MIDIChannelPrefix,
    EndOfTrack,
    SetTempo,
    SMTPEOffset,
    TimeSignature,
    KeySignature,
    SequencerSpecificMetaEvent,
    Unknown { event_type: u8 },
}
impl MetaEvent {
    pub fn new(event_type: u8) -> MetaEvent {
        match event_type {
            0x00 => MetaEvent::SequenceNumber,
            0x01 => MetaEvent::TextEvent,
            0x02 => MetaEvent::CopyrightNotice,
            0x03 => MetaEvent::SequenceOrTrackName,
            0x04 => MetaEvent::InstrumentName,
            0x05 => MetaEvent::Lyric,
            0x06 => MetaEvent::Marker,
            0x07 => MetaEvent::CuePoint,
            0x20 => MetaEvent::MIDIChannelPrefix,
            0x2F => MetaEvent::EndOfTrack,
            0x51 => MetaEvent::SetTempo,
            0x54 => MetaEvent::SMTPEOffset,
            0x58 => MetaEvent::TimeSignature,
            0x59 => MetaEvent::KeySignature,
            0x7F => MetaEvent::SequencerSpecificMetaEvent,
            _ => MetaEvent::Unknown { event_type },
        }
    }
}
impl MessageTool for MetaEvent {
    fn binary(&self) -> Vec<u8> {
        use messages::MetaEvent::*;
        vec![
            self.status_byte(),
            match *self {
                SequenceNumber => 0x00,
                TextEvent => 0x01,
                CopyrightNotice => 0x02,
                SequenceOrTrackName => 0x03,
                InstrumentName => 0x04,
                Lyric => 0x05,
                Marker => 0x06,
                CuePoint => 0x07,
                MIDIChannelPrefix => 0x20,
                EndOfTrack => 0x2F,
                SetTempo => 0x51,
                SMTPEOffset => 0x54,
                TimeSignature => 0x58,
                KeySignature => 0x59,
                SequencerSpecificMetaEvent => 0x7F,
                Unknown { event_type } => event_type,
            },
        ]
    }
    fn len(&self) -> usize {
        2
    }
    fn status_byte(&self) -> u8 {
        0xff
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum MidiEvent {
    NoteOff { ch: u8, note: u8, velocity: u8 },
    NoteOn { ch: u8, note: u8, velocity: u8 },
    PolyphonicKeyPressure { ch: u8, note: u8, velocity: u8 },
    ControlChange { ch: u8, control: u8, data: u8 },
    ProgramChange { ch: u8, program: u8 },
    ChannelPressure { ch: u8, pressure: u8 },
    PitchBendChange { ch: u8, data: i16 },
    Unknown { ch: u8 },
}
pub struct MidiEventBuilder {
    status: u8,
    shortage: u8,
    data: Vec<u8>,
}
impl MidiEventBuilder {
    pub fn new(status: u8) -> MidiEventBuilder {
        MidiEventBuilder {
            status: status,
            data: Vec::new(),
            shortage: match status & 0xf0 {
                0x80...0xb0 | 0xe0 => 2,
                0xc0 | 0xd0 => 1,
                _ => 0,
            },
        }
    }
    pub fn push(&mut self, data: u8) {
        if self.shortage > 0 {
            self.data.push(data);
            self.shortage -= 1;
        }
    }
    pub fn shortage(&self) -> u8 {
        self.shortage
    }
    pub fn build(&self) -> MidiEvent {
        match self.status & 0xf0 {
            0x80 => {
                MidiEvent::NoteOff {
                    ch: self.status & 0x0f,
                    note: self.data[0],
                    velocity: self.data[1],
                }
            }
            0x90 => {
                MidiEvent::NoteOn {
                    ch: self.status & 0x0f,
                    note: self.data[0],
                    velocity: self.data[1],
                }
            }
            0xa0 => {
                MidiEvent::PolyphonicKeyPressure {
                    ch: self.status & 0x0f,
                    note: self.data[0],
                    velocity: self.data[1],
                }
            }
            0xb0 => {
                MidiEvent::ControlChange {
                    ch: self.status & 0x0f,
                    control: self.data[0],
                    data: self.data[1],
                }
            }
            0xc0 => {
                MidiEvent::ProgramChange {
                    ch: self.status & 0x0f,
                    program: self.data[0],
                }
            }
            0xd0 => {
                MidiEvent::ChannelPressure {
                    ch: self.status & 0x0f,
                    pressure: self.data[0],
                }
            }
            0xe0 => {
                let lsb = self.data[0] as u16;
                let msb = (self.data[1] as u16) << 8;
                MidiEvent::PitchBendChange {
                    ch: self.status & 0x0f,
                    data: (msb & lsb) as i16 - 8192,
                }
            }
            _ => MidiEvent::Unknown { ch: self.status & 0x0f },
        }
    }
}
impl MessageTool for MidiEvent {
    fn binary(&self) -> Vec<u8> {
        use messages::MidiEvent::*;
        match *self {
            NoteOff { note, velocity, .. } => vec![self.status_byte(), note, velocity],
            NoteOn { note, velocity, .. } => vec![self.status_byte(), note, velocity],
            PolyphonicKeyPressure { note, velocity, .. } => {
                vec![self.status_byte(), note, velocity]
            }
            ControlChange { control, data, .. } => vec![self.status_byte(), control, data],
            ProgramChange { program, .. } => vec![self.status_byte(), program],
            ChannelPressure { pressure, .. } => vec![self.status_byte(), pressure],
            MidiEvent::PitchBendChange { data, .. } => {
                let pitch_bend: u16 = (data + 8192) as u16;
                vec![
                    self.status_byte(),
                    (pitch_bend >> 7) as u8,
                    (pitch_bend & 0b1111111) as u8,
                ]
            }
            MidiEvent::Unknown { .. } => vec![self.status_byte()],
        }
    }
    fn len(&self) -> usize {
        use messages::MidiEvent::*;
        match *self {
            NoteOff { .. } |
            NoteOn { .. } |
            PolyphonicKeyPressure { .. } |
            ControlChange { .. } |
            PitchBendChange { .. } => 3,
            ProgramChange { .. } |
            ChannelPressure { .. } => 2,
            Unknown { .. } => 1,
        }
    }
    fn status_byte(&self) -> u8 {
        use messages::MidiEvent::*;
        match *self {
            NoteOff { ch, .. } => 0x80 | (ch & 0x0f),
            NoteOn { ch, .. } => 0x90 | (ch & 0x0f),
            PolyphonicKeyPressure { ch, .. } => 0xa0 | (ch & 0x0f),
            ControlChange { ch, .. } => 0xb0 | (ch & 0x0f),
            ProgramChange { ch, .. } => 0xc0 | (ch & 0x0f),
            ChannelPressure { ch, .. } => 0xd0 | (ch & 0x0f),
            PitchBendChange { ch, .. } => 0xe0 | (ch & 0x0f),
            Unknown { ch } => 0x80 | (ch & 0x0f),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum SysExEvent {
    F0,
    F7,
    Unknown { status: u8 },
}
impl SysExEvent {
    pub fn new(status: u8) -> SysExEvent {
        use messages::SysExEvent::*;
        match status {
            0xF0 => F0,
            0xF7 => F7,
            _ => Unknown { status: status },
        }
    }
}
impl MessageTool for SysExEvent {
    fn binary(&self) -> Vec<u8> {
        vec![self.status_byte()]
    }
    fn len(&self) -> usize {
        1
    }
    fn status_byte(&self) -> u8 {
        match *self {
            SysExEvent::F0 { .. } => 0xf0,
            SysExEvent::F7 { .. } => 0xf7,
            SysExEvent::Unknown { status, .. } => status,
        }
    }
}
