extern crate ghakuf;

use ghakuf::messages::*;
use ghakuf::reader::*;
use ghakuf::writer::*;

fn main() {
    // build example
    let mut writer = Writer::new();
    writer.running_status(true);
    let tempo: u32 = 60 * 1000000 / 102; //bpm:102
    writer.push(Message::MetaEvent {
        delta_time: 0,
        event: MetaEvent::SetTempo,
        data: [(tempo >> 16) as u8, (tempo >> 8) as u8, tempo as u8].to_vec(),
    });
    writer.push(Message::MetaEvent {
        delta_time: 0,
        event: MetaEvent::EndOfTrack,
        data: Vec::new(),
    });
    writer.push(Message::TrackChange);
    writer.push(Message::MidiEvent {
        delta_time: 0,
        event: MidiEvent::NoteOn {
            ch: 0,
            note: 0x3c,
            velocity: 0x7f,
        },
    });
    writer.push(Message::MidiEvent {
        delta_time: 192,
        event: MidiEvent::NoteOn {
            ch: 0,
            note: 0x40,
            velocity: 0,
        },
    });
    writer.push(Message::MetaEvent {
        delta_time: 0,
        event: MetaEvent::EndOfTrack,
        data: Vec::new(),
    });
    writer.write("examples/example.mid").unwrap();

    // parse example
    let mut read_messages: Vec<Message> = Vec::new();
    let mut handler = HogeHandler {messages: &mut read_messages};
    let mut reader = Reader::new(&mut handler, "examples/example.mid").unwrap();
    reader.read().unwrap();
}

struct HogeHandler<'a> {
    messages: &'a mut Vec<Message>
}
impl<'a> Handler for HogeHandler<'a> {
    fn header(&mut self, format: u16, track: u16, time_base: u16) {
        println!(
            "SMF format: {}, track: {}, time base: {}",
            format,
            track,
            time_base
        );
    }
    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
        println!(
            "delta time: {:>4}, Meta event: {}, data: {:?}",
            delta_time,
            event,
            data
        );
        self.messages.push(
            Message::MetaEvent {
                delta_time: delta_time,
                event: event.clone(),
                data: data.clone(),
            }
        );
    }
    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
        println!(
            "delta time: {:>4}, MIDI event: {}",
            delta_time,
            event,
        );
        self.messages.push(
            Message::MidiEvent {
                delta_time: delta_time,
                event: event.clone()
            }
        );
    }
    fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
        println!(
            "delta time: {:>4}, System Exclusive Event: {}, data: {:?}",
            delta_time,
            event,
            data
        );
        self.messages.push(
            Message::SysExEvent {
                delta_time: delta_time,
                event: event.clone(),
                data: data.clone(),
            }
        );
    }
    fn track_change(&mut self) {
        // Excepts first track change (from format chunk to data chunk) 
        if self.messages.len() > 0 {
            println!("Track change occcurs!");
            self.messages.push(Message::TrackChange)
        }
    }
}
