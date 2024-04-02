pub mod types;
pub mod parser;
pub mod util;
use std::borrow::Cow;

pub use types::*;
pub use util::*;
pub use parser::*;
pub trait SongResolver{
    fn get_song(&self, id: &str) -> Result<Cow<Song>,ResourceError>;
    fn compile_entry(&self, entry: &SonglistEntry) -> Result<CompiledSong, LSTLinkErrorKind> {
        if let Some(song) = &entry.inline_data {
            //Maybe not clone?
            let mut out = song.clone();
            if let Some(tonic_to) = entry.tonic {
                let delta = delta_tonality(song.tonality, tonic_to);
                out.transpose(delta);
            }
            let mut out: CompiledSong = out.into();
            out.joined = entry.joined;
            return Ok(out);
        }

        if let Ok(song) = self.get_song(&entry.id_file) {
            let tonality = entry.tonic.unwrap_or(song.tonality);
            let delta = delta_tonality(song.tonality, tonality);
            let name = entry.rename.as_ref().unwrap_or(&song.name).to_string();

            let mut out = CompiledSong {
                name,
                bpm: song.bpm,
                tonality,
                id : Some(entry.id_file.clone()),
                source_tonality : song.tonality,
                joined: entry.joined,
                headless: false,
                sections: Vec::new(),
            };

            if let Some(named) = &entry.named_order {
                let order = song.orders.get(named);
                if let Some(index) = order {
                    let secs = index.iter().map(|i| song.sections[*i].clone());
                    out.sections.extend(secs);
                } else {
                    return Err(LSTLinkErrorKind::OrderNotFound(named.clone()));
                }
            } else if let Some(section_names) = &entry.explicit_order {
                for oelem in section_names {
                    match oelem {
                        OrderElement::Section(name) => {
                            if let Some(i) = song.section_names.get(name) {
                                out.sections.push(song.sections[*i].clone());
                            } else {
                                return Err(LSTLinkErrorKind::SectionNotFound(name.clone()));
                            }
                        }
                        OrderElement::Annotation(annotation) => {
                            out.sections.push(SongBlock::Annotation(annotation.clone()));
                        }
                    }
                }
            } else if let Some(default) = song
                .orders
                .get("default")
                .or_else(|| song.orders.get("main"))
            {
                for i in default {
                    out.sections.push(song.sections[*i].clone());
                }
            } else {
                out.sections = song
                    .sections
                    .iter()
                    .filter(|sblock| {
                        //TODO: we are ignoring annotations
                        if let SongBlock::Section(section) = sblock {
                            !section.name.ends_with('~')
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();
            }
            if delta != 0 {
                out.transpose(delta);
            }
            Ok(out)
        } else {
            Err(LSTLinkErrorKind::SongNotFound(entry.id_file.clone()))
        }
    }
    fn compile_list(&self, list: &Songlist) -> (Vec<CompiledSong>, Vec<LSTLinkError>) {
        let mut out: Vec<CompiledSong> = Vec::new();
        let mut err_out: Vec<LSTLinkError> = Vec::new();
        let mut last_is_valid = false;
        for element in list {
            match element {
                SonglistElement::Entry(entry) => match self.compile_entry(entry) {
                    Ok(mut song) => {
                        if song.joined && last_is_valid {
                            out.last_mut().unwrap().sections.append(&mut song.sections);
                        } else {
                            out.push(song);
                        }
                        last_is_valid = true;
                    }
                    Err(kind) => {
                        err_out.push(LSTLinkError {
                            kind,
                            context: Some(entry.id_file.clone()),
                            line: entry.line,
                            loc: (0..0), //TODO
                            loc16: (entry.utf16_offset..entry.utf16_offset)
                        });
                        last_is_valid = false;
                    }
                },
                SonglistElement::Annotation(att) => {
                    out.push(CompiledSong {
                        bpm: None,
                        joined: false,
                        name: String::new(),
                        id: None,
                        tonality: Tonality::default(),
                        source_tonality : Tonality::default(),
                        headless: true,
                        sections: vec![SongBlock::Annotation(att.clone())],
                    });
                    continue;
                }
            };
        }
        (out, err_out)
    }
}


pub type AsyncGetSongFn = Box<dyn Fn(&str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Song,ResourceError>>>>>;

pub async fn async_compile_entry(resolver : &AsyncGetSongFn,  entry: &SonglistEntry) -> Result<CompiledSong, LSTLinkErrorKind>{
    if let Some(song) = &entry.inline_data {
        //Maybe not clone?
        let mut out = song.clone();
        if let Some(tonic_to) = entry.tonic {
            let delta = delta_tonality(song.tonality, tonic_to);
            out.transpose(delta);
        }
        let mut out: CompiledSong = out.into();
        out.joined = entry.joined;
        return Ok(out);
    }

    if let Ok(song) = resolver(&entry.id_file).await {
        let tonality = entry.tonic.unwrap_or(song.tonality);
        let delta = delta_tonality(song.tonality, tonality);
        let name = entry.rename.as_ref().unwrap_or(&song.name).to_string();

        let mut out = CompiledSong {
            name,
            bpm: song.bpm,
            tonality,
            id : Some(entry.id_file.clone()),
            source_tonality : song.tonality,
            joined: entry.joined,
            headless: false,
            sections: Vec::new(),
        };

        if let Some(named) = &entry.named_order {
            let order = song.orders.get(named);
            if let Some(index) = order {
                let secs = index.iter().map(|i| song.sections[*i].clone());
                out.sections.extend(secs);
            } else {
                return Err(LSTLinkErrorKind::OrderNotFound(named.clone()));
            }
        } else if let Some(section_names) = &entry.explicit_order {
            for oelem in section_names {
                match oelem {
                    OrderElement::Section(name) => {
                        if let Some(i) = song.section_names.get(name) {
                            out.sections.push(song.sections[*i].clone());
                        } else {
                            return Err(LSTLinkErrorKind::SectionNotFound(name.clone()));
                        }
                    }
                    OrderElement::Annotation(annotation) => {
                        out.sections.push(SongBlock::Annotation(annotation.clone()));
                    }
                }
            }
        } else if let Some(default) = song
            .orders
            .get("default")
            .or_else(|| song.orders.get("main"))
        {
            for i in default {
                out.sections.push(song.sections[*i].clone());
            }
        } else {
            out.sections = song
                .sections
                .iter()
                .filter(|sblock| {
                    //TODO: we are ignoring annotations
                    if let SongBlock::Section(section) = sblock {
                        !section.name.ends_with('~')
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
        }
        if delta != 0 {
            out.transpose(delta);
        }
        Ok(out)
    } else {
        Err(LSTLinkErrorKind::SongNotFound(entry.id_file.clone()))
    }
}

pub async fn async_compile_list(resolver : AsyncGetSongFn,  list: &Songlist) -> (Vec<CompiledSong>, Vec<LSTLinkError>){
    let mut out: Vec<CompiledSong> = Vec::new();
        let mut err_out: Vec<LSTLinkError> = Vec::new();
        let mut last_is_valid = false;
        for element in list {
            match element {
                SonglistElement::Entry(entry) => match async_compile_entry(&resolver,entry).await {
                    Ok(mut song) => {
                        if song.joined && last_is_valid {
                            out.last_mut().unwrap().sections.append(&mut song.sections);
                        } else {
                            out.push(song);
                        }
                        last_is_valid = true;
                    }
                    Err(kind) => {
                        err_out.push(LSTLinkError {
                            kind,
                            context: Some(entry.id_file.clone()),
                            line: entry.line,
                            loc: (0..0), //TODO
                            loc16: (entry.utf16_offset..entry.utf16_offset)
                        });
                        last_is_valid = false;
                    }
                },
                SonglistElement::Annotation(att) => {
                    out.push(CompiledSong {
                        bpm: None,
                        joined: false,
                        name: String::new(),
                        id : None,
                        tonality: Tonality::default(),
                        source_tonality : Tonality::default(),
                        headless: true,
                        sections: vec![SongBlock::Annotation(att.clone())],
                    });
                    continue;
                }
            };
        }
        (out, err_out)
}