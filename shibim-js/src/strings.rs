use std::{collections::HashMap, fmt::write};
use shibim_parse::ChordKeyword;
use unicode_normalization::UnicodeNormalization;
use shibim_parse::{Song, Line, SongBlock, LyricEvent};
use core::fmt::Display;
use lazy_static;
#[derive(Debug, Clone, Copy)]
pub enum Lang{
    EN,
    ES
}

pub fn normalize_text(to_norm : &str, lang : Lang) -> String{
    match lang{
        Lang::EN =>
            to_norm
            .chars()
            .filter_map( |x|{
                if x.is_alphabetic() || x == '\''{
                    Some(x)
                }else if x.is_whitespace(){
                    Some(' ')
                }else{
                    None
                }
            })
            .scan(' ',|acc,c|{
                if *acc==c && c ==' '{
                    Some(None)
                }else{
                    *acc = c;
                    Some(Some(c))
                }
            })
            .filter_map(|x|x)
            .flat_map(|c|c.to_lowercase())
            .collect::<String>()
        ,
        Lang::ES=> 
            to_norm
            .nfd()
            .filter_map( |x|{
                if x.is_alphabetic() || x == '\u{0303}' {
                    Some(x)
                }else if x.is_whitespace(){
                    Some(' ')
                }else{
                    None
                }
            })
            .scan(' ',|acc,c|{
                if *acc==c && c ==' '{
                    Some(None)
                }else{
                    *acc = c;
                    Some(Some(c))
                }
            })
            .filter_map(|x|x)
            .nfc()
            .flat_map(|c|c.to_lowercase())
            .collect::<String>()
    }
}


pub struct SongNormTextWrite<'i>(pub &'i Song,pub Lang);
pub struct LineNormTextWrite<'i>(pub &'i Line,pub Lang);


impl<'i> Display for SongNormTextWrite<'i>{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for sblock in &self.0.sections{
            if let SongBlock::Section(section) = sblock{
                for subs in &section.subsections{
                    for line in &subs.lines{
                        write!(f,"{}",LineNormTextWrite{0:line,1:self.1})?;     
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'i> Display for LineNormTextWrite<'i>{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.0 {
            Line::Lyrics(m)=>{
                for evt in m.iter().flatten().flatten(){
                    if let LyricEvent::LyricText(evt) = evt{
                        let text = normalize_text(evt,self.1);
                        if text.len() > 0 {
                            write!(f,"{}",text)?;
                        }else if evt.find(char::is_whitespace) != None{
                            write!(f," ")?;
                        }
                    }
                }
                write!(f," ")?;
                Ok(())
            },
            Line::Mixed(m)=>{
                for (_,block) in m.iter().flatten(){
                    for evt in block{
                        if let LyricEvent::LyricText(evt) = evt{
                            let text = normalize_text(evt,self.1);
                            if text.len() > 0 {
                                write!(f,"{}",text)?;
                            }else if evt.find(char::is_whitespace) != None{
                                write!(f," ")?;
                            }
                        }
                    }
                }
                write!(f," ")?;
                Ok(())
            }
            _ => Ok(())
        }
    }
}

lazy_static::lazy_static! {
    pub static ref EN: HashMap<&'static str, &'static str> = {
        HashMap::from([
            ("Mon", "Mon"),
            ("Tue", "Tue"),
            ("Wed", "Wed"),
            ("Thu", "Thu"),
            ("Fri","Fri"),
            ("Sat","Sat"),
            ("Sun","Sat")
        ])
    };
    pub static ref CHORD_KEYWORDS_NAMES: HashMap<ChordKeyword, &'static str> = {
        HashMap::from([
            (ChordKeyword::Sus2,"sus2"),
            (ChordKeyword::Sus4,"sus4"),
            (ChordKeyword::Add2,"add2"),
            (ChordKeyword::Add4,"add4"),
            (ChordKeyword::Add9,"add9"),
            (ChordKeyword::Add11,"add11"),
            (ChordKeyword::Dim,"dim"),
            (ChordKeyword::Aug,"aug"),
            (ChordKeyword::Maj,"Δ"),
            (ChordKeyword::K5,"5"),
            (ChordKeyword::K6,"6"),
            (ChordKeyword::K7,"7"),
            (ChordKeyword::K9,"9"),
            (ChordKeyword::K11,"11"),
            (ChordKeyword::K13,"13"),
            (ChordKeyword::K69,"6/9"),
            (ChordKeyword::Sus,"sus")
        ])
    };
}