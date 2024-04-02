use crate::types::*;
pub fn is_altered(a : NoteHeight) -> bool{
    let a  = a % 12;
    if a < 5 {
        a & 1 == 1
    }else{
        a & 1 == 0
    }

}
fn get_rel_tonic(a : Tonality) -> u8{
    if let TonicKind::Minor = a.kind{
        (a.tonic + 3) %12
    }else{
        a.tonic
    }
}
pub fn delta_tonality(from : Tonality, to : Tonality)->u8{
    (get_rel_tonic(to) + 12 - get_rel_tonic(from)) % 12
}
pub fn get_default_use_flat(a : Tonality)->bool{
    let rel_tonic = get_rel_tonic(a);
    matches!(rel_tonic, 3|5|8|10)
}

pub trait Transposable{
    fn transpose(&mut self, delta : u8);
}

impl Transposable for CompiledSong{
    fn transpose(&mut self,delta: u8){
        for sec in &mut self.sections{
            sec.transpose(delta);
        }
    }
}
impl Transposable for Song{
    fn transpose(&mut self, delta : u8) {
        for sec in &mut self.sections{
            sec.transpose(delta);
        }
        self.tonality.tonic = (delta + self.tonality.tonic) % 12
    }
}
impl Transposable for SongBlock{
    fn transpose(&mut self, delta : u8) {
        match self {
            SongBlock::Section(section)=> section.transpose(delta),
            _ => {}
        }
    }
}
impl Transposable for Section {
    fn transpose(&mut self, delta : u8){
        for subs in &mut self.subsections{
            subs.transpose(delta);
        }
    }
}
impl Transposable for Subsection {
    fn transpose(&mut self, delta : u8) {
        for line in &mut self.lines{
            line.transpose(delta);
        }
    }
}

impl Transposable for Line {
    fn transpose(&mut self, delta : u8) {
        match self{
            Line::Chords(c)=>{
                c.iter_mut()
                    .flatten().flatten()
                    .for_each(|evt|evt.transpose(delta));
            }
            Line::Mixed(c)=>{
                c.iter_mut().flatten()
                    .for_each(|(evts,_)|
                        evts.iter_mut()
                        .for_each(|evt|evt.transpose(delta))
                    );
            }
            _=>{}
        }
    }
}

impl Transposable for MusicEvent {
    fn transpose(&mut self, delta : u8) {
        match self {
            MusicEvent::ChordEvent(c)=>{
                c.transpose(delta);
            }
            MusicEvent::MelodyEvent(m)=>{
                for evt in m{
                    *evt = (*evt + 12 + delta) % 12;
                }
            }
            _=>{}
        }
    }
}

impl Transposable for ChordEvent {
    fn transpose(&mut self, delta : u8) {
        self.root = (self.root + 12 + delta) % 12;
        if let Some(bass) = &mut self.bass{
            *bass = (*bass + 12 + delta) % 12; 
        }
    }
}
