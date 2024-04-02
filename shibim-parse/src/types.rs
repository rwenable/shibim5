use std::collections::HashMap;
use std::collections::HashSet;
use  std::iter::{Flatten,FilterMap,FlatMap};
use std::borrow::Cow;
use std::fmt::Display;
use thiserror::Error;
pub type NoteHeight = u8;
pub const CHAR_TONIC_VALUES:[u8;7] = [9,11,0,2,4,5,7];
pub const SHARP_TONIC_NAMES:[&str;12] = ["C","C","D","D","E","F","F","G","G","A","A","B"];
pub const FLAT_TONIC_NAMES:[&str;12] = ["C","D","D","E","E","F","G","G","A","A","B","B"];


#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum TonicKind{
    Minor = 1,
    Major = 0,
    Undefined = 2
}

impl TryFrom<i32> for TonicKind {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x ==TonicKind::Minor as i32 => Ok(TonicKind::Minor),
            x if x ==TonicKind::Major as i32 => Ok(TonicKind::Major),
            x if x ==TonicKind::Undefined as i32 => Ok(TonicKind::Undefined),
            
            _ => Err(())
        }
    }
}

#[derive(Debug,Clone,Copy,Default)]
pub struct Tonality{
    pub tonic : NoteHeight,
    pub kind : TonicKind
}
#[derive(Debug,Clone,Copy)]
pub enum ChordKind{
        Minor,
        Major,
        Undefined
}

#[derive(Debug,Clone,Default)]
pub struct Song{
    pub name: String,
    pub tonality : Tonality,
    pub bpm : Option<f32>,
    pub id : Option<String>,
    pub sections : Vec<SongBlock>,
    pub categories : HashSet<String>,
    pub metadata : HashMap<String,String>,
    pub section_names : HashMap<String,usize>,
    pub orders : HashMap<String,Vec<usize>>
}

#[derive(Debug,Clone)]
pub struct CompiledSong{
    pub name: String,
    pub tonality : Tonality,
    pub source_tonality : Tonality,
    pub id : Option<String>,
    pub bpm : Option<f32>,
    pub joined : bool,
    pub headless : bool,
    pub sections : Vec<SongBlock>
}

#[derive(Debug,Clone)]
pub enum SongBlock{
    Section(Section),
    Annotation(String)
}
pub struct SongRef<'i> {
    pub name : &'i str,
    pub tonality : Tonality,
    pub source_tonality : Tonality,
    pub id : Option<&'i str>,
    pub bpm : Option<f32>,
    pub sections : &'i Vec<SongBlock>,
    pub headless: bool
}
#[derive(Debug,Clone)]
pub struct SectionName{
    pub kind : String,
    pub number : u16,
    pub version : String,
}

#[derive(Debug,Clone,Default)]
pub struct Section{
        pub name : String,
        pub description : String,
        pub local_tonality : Option<Tonality>,
        pub subsections : Vec<Subsection>,
        pub has_abc : bool,
    //pub metadata : HashMap<String,String>
}

#[derive(Debug,Clone,Default)]
pub struct Subsection{
        pub metadata : HashMap<String,String>,
        pub lines : Vec<Line>,
        pub abc : Option<String>
}

//Line: Vector of possibly empty measures
//Measure: Vector of blocks
//Block: Vector of events (or a tuple of vectors)
type MixedEventList = (Vec<MusicEvent>,Vec<LyricEvent>);

type LyricMeasure = Vec<Vec<LyricEvent>>;
type ChordMeasure = Vec<Vec<MusicEvent>>;
type MixedMeasure = Vec<MixedEventList>;

type LyricLine = Vec<LyricMeasure>;
type ChordLine = Vec<ChordMeasure>;
type MixedLine = Vec<MixedMeasure>;




#[derive(Debug,Clone)]
pub enum Line{
        Lyrics  (LyricLine),
        Chords  (ChordLine),
        Mixed   (MixedLine)
}
#[derive(Debug,Clone)]
pub enum LyricEvent{
        LyricText(String),
        LyricBreak
}

#[derive(Debug,Clone)]
pub enum MusicEvent{
    ChordEvent(ChordEvent),
    RepeatMeasure,
    StartRepeat,
    EndRepeat,
    OpenParen,
    CloseParen,
    NumberedMeasure(u16),
    Annotation(String),
    MelodyEvent(Vec<NoteHeight>)
}

#[derive(Debug,Clone)]
// We canmelo pass attributes through to generated types with archive_attr
pub enum ChordModifier{
    Keyword(ChordKeyword),
    Alteration(ChordAlteration)
}


#[derive(Debug,Clone)]
pub struct ChordEvent{
        pub root : NoteHeight,
        pub bass : Option<NoteHeight>,
        pub kind : ChordKind,
        pub modifiers : Vec<ChordModifier>,
        pub time : Option<TimeOffset>
}

#[derive(Debug,Clone,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub enum ChordKeyword{
        Sus2,
        Sus4,
        Add2,
        Add4,
        Add9,
        Add11,
        Maj,
        K6,
        K5,
        K7,
        K9,
        K11,
        K13,
        K69,
        Aug,
        Dim,
    Sus
}

#[derive(Debug,Clone)]
pub enum ChordAlterationKind{
        Flat,
        Sharp,
        No
}

#[derive(Debug,Clone)]
pub struct ChordAlteration{
        pub kind : ChordAlterationKind,
        pub degree : u8
}


#[derive(Debug,Clone)]
pub struct TimeOffset{
        pub beat: i8,
        pub num : u8,
        pub den : u8
}

#[derive(Debug,Default,Clone)]
pub struct SonglistEntry{
    pub id_file : String,
    pub rename : Option<String>,
    pub tonic : Option<Tonality>,
    pub explicit_order : Option<Vec<OrderElement>>,
    pub named_order : Option<String>,
    pub joined : bool,
    pub inline_data : Option<Song>,
    pub line : usize,
    pub utf16_offset : usize //For error reporting
}
#[derive(Debug,Clone)]
pub enum OrderElement{
    Section(String),
    Annotation(String)
}

#[derive(Debug,Clone)]
pub enum SonglistElement{
    Entry(SonglistEntry),
    Annotation(String)
}

pub type Songlist = Vec<SonglistElement>;

pub struct SongIndexEntry<'a>{
    pub name : Cow<'a, str>,
    pub subtitle : Option<Cow<'a, str>>,
    pub tonality : Tonality,
    pub href : Cow<'a, str>,
    pub section_names : Vec<(Cow<'a, str>,Cow<'a, str>)>
}

pub struct SonglistIndexEntry<'a>{
    pub name : Cow<'a, str>,
    pub songs : Vec<Cow<'a, str>>,
    pub href : Cow<'a, str>
}

pub struct SongTextWrite<'i>(&'i Song);
pub struct LineTextWrite<'i>(&'i Line);


impl Song{
    pub fn display(&self) -> SongTextWrite{
        SongTextWrite(self)
    }
    pub fn all_lines<'i>(&'i self) -> impl Iterator<Item=&Line> + 'i{
        return self
            .sections
            .iter()
            .filter_map(|songblock|songblock.section())
            .flat_map(|section|&section.subsections)
            .flat_map(|subsection|&subsection.lines);
    }
    pub fn all_text<'i>(&'i self) -> impl Iterator<Item=&str>{
        self.all_lines().flat_map(|m|
            m.text_iter()
            .chain(std::iter::once("\n"))
        )
    }
}
impl SongBlock{
    pub fn section(&self)->Option<&Section>{
        match self {
            SongBlock::Section(s) => Some(s),
            _ => None
        }
    }
    pub fn into_section(self)->Option<Section>{
        match self {
            SongBlock::Section(s) => Some(s),
            _ => None
        }
    }
    pub fn section_mut(&mut self)->Option<&mut Section>{
        match self {
            SongBlock::Section(s) => Some(s),
            _ => None
        }
    }
    pub fn section_unwrap(&self)->&Section{
        match self {
            SongBlock::Section(s) => s,
            _ => panic!("Can't use annotation as section.")
        }
    }
    pub fn section_unwrap_mut(&mut self)->&mut Section{
        match self {
            SongBlock::Section(s) => s,
            _ => panic!("Can't use annotation as section.")
        }
    }
    pub fn annotation_unwrap_mut(&mut self)->&mut String{
        match self {
            SongBlock::Annotation(s) => s,
            _ => panic!("Can't use section as annotation.")
        }
    }
}
fn lyric_line_text_filter_fn(line : &LyricEvent)->Option<&str>{
    match line{
        LyricEvent::LyricText(st) => Some(st.as_str()),
        _=> None
    }
}
fn second(a : &MixedEventList) -> &Vec<LyricEvent>{
    &a.1
}

pub enum LineTextIterEnum<'i>{
    Lyric (FilterMap<Flatten<Flatten<std::slice::Iter<'i,LyricMeasure>>>,fn(&LyricEvent)->Option<&str>>),
    Chord,
    Mixed (FilterMap<FlatMap<Flatten<std::slice::Iter<'i,MixedMeasure>>,&'i Vec<LyricEvent>,fn(&MixedEventList)->&Vec<LyricEvent>>,fn(&LyricEvent)->Option<&str>>),
}
impl<'i> Iterator for LineTextIterEnum<'i>{
    type Item = &'i str;
    fn next(&mut self) -> Option<Self::Item> {
        match self{
            LineTextIterEnum::Chord=>None,
            LineTextIterEnum::Lyric(it)=>it.next(),
            LineTextIterEnum::Mixed(it)=>it.next()
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>){
        match self{
            LineTextIterEnum::Chord=>(0,Some(0)),
            LineTextIterEnum::Lyric(it)=>it.size_hint(),
            LineTextIterEnum::Mixed(it)=>it.size_hint()
        }
    }
}
impl Line{
    pub fn text(&self) -> LineTextWrite{
        LineTextWrite(self)
    }
    pub fn text_iter(&self) -> LineTextIterEnum{
        match &self{
            Line::Chords(_)=>{
                LineTextIterEnum::Chord
            },
            Line::Mixed(arr)=>{
                LineTextIterEnum::Mixed(
                    arr.iter().flatten().flat_map(second as fn(&MixedEventList)->&Vec<LyricEvent>)
                    .filter_map(lyric_line_text_filter_fn as fn(&LyricEvent)->Option<&str>)
                )
            },
            Line::Lyrics(arr)=>{
                LineTextIterEnum::Lyric(
                    arr.iter().flatten().flatten().filter_map(lyric_line_text_filter_fn as fn(&LyricEvent)->Option<&str>)
                )
            }
        }
    }
}
impl Default for SongBlock{
    fn default() -> Self {
        SongBlock::Section(Section::default())
    }
}
impl<'i> Display for SongTextWrite<'i>{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for sblock in &self.0.sections{
            if let SongBlock::Section(section) = sblock{
                for subs in &section.subsections{
                    for line in &subs.lines{
                        write!(f,"{}",line.text())?;     
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'i> Display for LineTextWrite<'i>{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.0 {
            Line::Lyrics(m)=>{
                for evt in m.iter().flatten().flatten(){
                    if let LyricEvent::LyricText(evt) = evt{
                        write!(f,"{}",evt)?;
                    }
                }
                writeln!(f)?;
                Ok(())
            },
            Line::Mixed(m)=>{
                for (_,block) in m.iter().flatten(){
                    for evt in block{
                        if let LyricEvent::LyricText(evt) = evt{
                            write!(f,"{}",evt)?;
                        }
                    }
                }
                writeln!(f)?;
                Ok(())
            }
            _ => Ok(())
        }
    }
}

impl std::convert::From<Song> for CompiledSong{
    fn from(item: Song) -> Self{
        CompiledSong{
            name : item.name,
            tonality : item.tonality,
            source_tonality : item.tonality,
            id : item.id,
            bpm : item.bpm,
            joined : false,
            headless : false,
            sections : item.sections
        }
    }
}
impl Default for TonicKind {
    fn default() -> Self {
        TonicKind::Major
    }
}
impl std::convert::From<&Song> for CompiledSong{
    fn from(item: &Song) -> Self{
        CompiledSong{
            name : item.name.clone(),
            tonality : item.tonality,
            source_tonality : item.tonality,
            id : item.id.clone(),   
            bpm : item.bpm,
            joined: false,
            sections : item.sections.clone(),
            headless : false
        }
    }
}


impl<'i> std::convert::From<&'i Song> for SongRef<'i>{
    fn from(item: &'i Song) -> Self{
        SongRef{
            name : &item.name,
            tonality : item.tonality,
            source_tonality : item.tonality,
            id : item.id.as_deref(),
            bpm : item.bpm,
            sections : &item.sections,
            headless: false
        }
    }
}

impl<'i> std::convert::From<&'i CompiledSong> for SongRef<'i>{
    fn from(item: &'i CompiledSong) -> Self{
        SongRef{
            name : &item.name,
            tonality : item.tonality,
            source_tonality : item.source_tonality,
            id : item.id.as_deref(),
            bpm : item.bpm,
            sections : &item.sections,
            headless : item.headless
        }
    }
}

#[derive(Error,Debug,Clone)]
#[error("Line {line}  '{0}' : {kind}", context.clone().unwrap_or_default() )]
pub struct  LocatedError<T>
    where T : std::error::Error{
    pub loc : std::ops::Range<usize>,
    pub loc16 : std::ops::Range<usize>,
    pub line : usize,
    pub kind : T,
    pub context : Option<String>,
}

pub type SHBParseError = LocatedError<SHBErrorKind>;
pub type LSTParseError = LocatedError<LSTErrorKind>;


#[derive(Debug,Error,Clone)] 
pub enum SHBErrorKind{
    #[error("Unknown music symbol '{0}'")]
    MalformedMusicEvent(String),
    #[error("Section name '{0}' already in use")]
    RepeatedSectionName(String),
    #[error("Order name '{0}' already in use")]
    RepeatedOrderName(String),
    #[error("Section '{0}' not found")]
    SectionNotFound(String),
    #[error("Unexpected character '{0}'")]
    UnexpectedChar(char),
    #[error("Section has no name")]
    MissingSectionID,
    #[error("Repeated chord separator")]
    RepeatedDot,
    #[error("Metadata argument has no value")]
    NoMetaValue(String),
    #[error("Empty metadata argument")]
    NoMetaName,
    #[error("Unknown tonic format")]
    WrongTonicFormat,
    #[error("Unknown order format '{0}'")]
    WrongOrderFormat(String),
    #[error("Unknown meta argument '{0}'")]
    UnknownMetaArgument(String)
}
#[derive(Debug,Error,Clone)]
pub enum LSTErrorKind{
    #[error("Entry with no name")]
    MissingSongName,
    #[error("Modifier has no name")]
    MissingMetaName,
    #[error("Unknown tonic format")]
    WrongTonicFormat,
    #[error("Unknown modifier '{0}'")]
    UnrecognizedMeta(String),
    #[error("Unexpected '{0}'")]
    UnexpectedChar(char),
    #[error("Error parsing inline song '{0}'")]
    InlineSongError(SHBErrorKind)
}

#[derive(Debug)]
pub enum ParseSongWarnings{
    RepeatedSectionName(String),
    SectionNotFound(String),
    UnNamed,
    WrongTonicFormat,
    NoTonic,
}

pub enum ParseListWarnings{
    SongNotFound(String),
    SongSectionsNotFound(Vec<String>),
    FirstJoined,
    UnknownSongArgs(String)
}
#[derive(Debug,Default,Clone)]
pub struct SongSessionInfo{
    pub cur_file : Option<std::path::PathBuf>,
    pub error_list : Vec<SHBParseError>
}

impl SongSessionInfo{
    pub fn new(cur_file : &std::path::Path)->Self{
        SongSessionInfo{
            cur_file : Some(cur_file.to_owned()),
            error_list : Vec::new()
        }
    }
    pub fn emit(&mut self,err : SHBParseError){
        if let Some(cur_file) = &self.cur_file{
            eprintln!("{}: {:?}",cur_file.display(),err);
        }else{
            eprintln!("[text source]: {:?}",err);
        }
        self.error_list.push(err);
    }
}

pub fn abs_tonality(tone : Tonality) -> NoteHeight{
    match tone.kind {
        TonicKind::Major => tone.tonic,
        TonicKind::Minor => tone.tonic+3,
        TonicKind::Undefined => tone.tonic
    }   
}

#[derive(Error,Debug)]
#[error("Error loading resource '{handle}' : '{}''",detail.as_ref().map(|x|x.to_string()).unwrap_or_default())]
pub struct ResourceError{
    pub handle : String,
    pub exists : bool,
    pub detail : Option<Box<dyn std::error::Error>>
}

#[derive(Error,Debug)]
#[error("Error loading resource '{file}'")]
pub struct FileError<T>{
    pub file : String,
    #[source]
    pub detail : T
}
impl<T> FileError<T>{
    pub fn new(err : T, handle : &str) -> FileError<T>{
        FileError{
            file : handle.to_string(),
            detail : err
        }
    }
}
//This is admittedly a bad, confusing design, i am sad
pub type LoadErrors<T> = FileError<ParseOrIOError<T>>;
pub type SHBError = LoadErrors<SHBParseError>;
#[derive(Error,Debug)]
pub enum LSTError{
    #[error(transparent)]
    LinkError(#[from] FileError<LSTLinkError>),
    #[error(transparent)]
    OpenError(#[from] LoadErrors<LSTParseError>)
}
pub type LSTLinkError = LocatedError<LSTLinkErrorKind>;
#[derive(Debug,Error)]

pub enum LSTLinkErrorKind{
    #[error("Song not found '{0}'")]
    SongNotFound(String),
    #[error("Section not found '{0}'")]
    SectionNotFound(String),
    #[error("Order not found '{0}'")]
    OrderNotFound(String),
}

#[derive(Error,Debug)]
#[error("Post-processing error, {msg}")]
pub struct VisitorError{
    pub msg : String,
    detail : Option<Box<dyn std::error::Error>>
}

#[derive(Error,Debug)]
pub enum ParseOrIOError<T>{
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Synax error(s) '{0:?}'")] //Todo, this is duct tape
    ParseError(Vec<T>)
}

