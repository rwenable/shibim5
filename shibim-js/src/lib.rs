mod utils;
mod html;
mod strings;
extern crate shibim_parse;
extern crate markup;
extern crate lazy_static;
extern crate js_sys;
use shibim_parse::{SHBParser, LSTParser, SongResolver, Section, async_compile_list, AsyncGetSongFn,ResourceError, SonglistElement};
use strings::SongNormTextWrite;
use wasm_bindgen::prelude::*;
use std::iter::FromIterator;

fn serialize_section_info<'i>(iter: impl Iterator<Item = &'i Section>) -> String {
    let mut str = String::new();
    for section in iter {
        str.reserve(section.name.len() + section.description.len() + 2);
        str.push_str(&section.name);
        str.push_str("|");
        str.push_str(&section.description);
        str.push_str("\n");
    }
    str
}

async fn get_song(id: String) -> Result<shibim_parse::Song,ResourceError> {
    let song_src = get_song_source_by_name(&id).await.map_err(|e|ResourceError{
        handle : id.clone(),
        exists : false,
        detail : None //TODO
    })?;
    let mut parser = SHBParser::default();
    let content = song_src.as_string().ok_or_else(||ResourceError{
        handle : id,
        exists : true,
        detail : None
    })?;
    parser.parse_str(&content);
    let (song,_errors) = parser.extract();
    Ok(song)
}
#[wasm_bindgen(getter_with_clone)]
pub struct ParsingError {
    pub line : usize,
    pub message : String,
    pub from : usize,
    pub to : usize
}

#[wasm_bindgen(getter_with_clone)]
pub struct SHB_Return {
    pub content : String,
    pub errors : JsValue
}

#[wasm_bindgen(getter_with_clone)]
pub struct SHB_Full_Return {
    pub content : String,
    pub plain : String,
    pub sections : String,
    pub errors : JsValue
    
}

#[wasm_bindgen(getter_with_clone)]
pub struct SHB_DB_Return {
    pub plain : String,
    pub sections : String,
    pub title : String,
    pub norm_title : String,
    pub subtitle : String,
    pub tonic : i8,
    pub tonic_kind : i8
}

#[wasm_bindgen(getter_with_clone)]
pub struct LST_Return {
    pub content : String,
    pub s_errors : JsValue,
    pub l_errors : JsValue
}

#[wasm_bindgen(js_namespace = ["window","__SHB_API__"])]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn get_song_source_by_name(name : &str) -> Result<JsValue, JsValue>;
}


#[wasm_bindgen]
pub fn shb_to_html(s : &str) -> JsValue{
    let mut parser = SHBParser::default();
    parser.parse_str(s);
    let (song,errors) = parser.extract();
    let errors  = errors.into_iter().map(|x|ParsingError{
        line : x.line,
        message : x.to_string(),
        from : x.loc16.start,
        to : x.loc16.end
    }).map(JsValue::from);
    let arr = js_sys::Array::from_iter(errors);
    SHB_Return{
        content : html::Song{
            song : &song
        }.to_string(),
        errors : arr.into()
    }.into()
    //Don't forget to .free() in js side!
}

#[wasm_bindgen]
pub fn shb_to_html_db(s : &str) -> JsValue{
    let mut parser = SHBParser::default();
    parser.parse_str(s);
    let (song,errors) = parser.extract();
    let errors  = errors.into_iter().map(|x|ParsingError{
        line : x.line,
        message : x.to_string(),
        from : x.loc16.start,
        to : x.loc16.end
    }).map(JsValue::from);
    let arr = js_sys::Array::from_iter(errors);
    let plain = SongNormTextWrite{0:&song,1:strings::Lang::ES}.to_string();
    let sections = serialize_section_info(song.sections.iter().filter_map(|x|x.section()));
    SHB_Full_Return{
        content : html::Song{
            song : &song
        }.to_string(),
        plain,
        sections,
        errors : arr.into()
    }.into()
}

#[wasm_bindgen]
pub fn shb_to_db_info(s : &str) -> JsValue{
    let mut parser = SHBParser::default();
    parser.parse_str(s);
    let (song,_errors) = parser.extract();
    SHB_DB_Return {
        plain : SongNormTextWrite{0:&song,1:strings::Lang::ES}.to_string(),
        sections : serialize_section_info(song.sections.iter().filter_map(|x|x.section())),
        title : song.name.clone(),
        norm_title : strings::normalize_text(&song.name,strings::Lang::ES),
        subtitle : song.metadata.get("subtitle")
            .map(|x| x.as_str())
            .unwrap_or_default().to_string(),
        tonic : song.tonality.tonic as i8,
        tonic_kind : song.tonality.kind as i8
    }.into()
}

#[wasm_bindgen]
pub async fn lst_to_html(s : &str) -> JsValue{
    let mut parser = LSTParser::default();
    //Thank you stack overflow (https://stackoverflow.com/questions/66769143)
    let func : AsyncGetSongFn = Box::new(move |name|{
        Box::pin(get_song(name.to_string()))
    });
    parser.parse_str(s);
    let (list, serrors) = parser.extract();
    let (list, lerrors) = async_compile_list(func,&list).await;
    let serrors = serrors.into_iter().map(|e|{
        ParsingError{
            line : e.line,
            message : e.to_string(),
            from : e.loc16.start,
            to : e.loc16.end
        }
    }).map(JsValue::from);
    let lerrors = lerrors.into_iter().map(|e|
        ParsingError{
            line : e.line,
            message : e.to_string(),
            from : e.loc16.start,
            to : e.loc16.end
        }
    ).map(JsValue::from);
    LST_Return{
        content : html::Songlist{
            list : &list
        }.to_string(),
        s_errors : js_sys::Array::from_iter(serrors).into(),
        l_errors : js_sys::Array::from_iter(lerrors).into()
    }.into()
}

#[wasm_bindgen]
pub async fn lst_to_array(s : &str) -> JsValue{
    let mut parser = LSTParser::default();
    parser.parse_str(s);
    let (list, _serrors) = parser.extract();
    let str_iter = list.into_iter().filter_map(|elem|
        if let SonglistElement::Entry(entry) = elem{
            if entry.inline_data.is_some(){
                None
            }else{
                Some(entry)
            }
        }else{
            None
        } 
    ).map(|entry|{
        JsValue::from(entry.id_file)
    });
    js_sys::Array::from_iter(str_iter).into()
}