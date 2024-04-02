use crate::types::*;

macro_rules! seek_cascade {

    ($s:expr, $key:expr => $value:expr ) => {
        {
            seek($s,$key).map(|ns|($value,ns))
        }
    };

    ($s:expr, $key:expr => $value:expr, $($k:expr => $v:expr),+ ) => {
        {
            if let Some(ns) = seek($s,$key){
                Some(($value,ns))
            }else{
                seek_cascade!($s,$($k => $v),+)
            }
        }
    };
}

#[derive(PartialEq, Eq, Debug)]
enum SHBParsingState {
    MetaStart,
    MetaArg,
    MetaVal,
    SectionOrAnnotationStart,
    SectionStart,
    SectionHeadId,
    SectionHeadDesc,
    SectionMetaArg,
    SectionMetaVal,
    SubsectionStart,
    LineStart,
    MeasureStart,
    BlockStart,
    MaybeLyricBlock(bool),
    TrueLyricBlock(char),
    LineEnd,
    SubsectionDelim(u8),
    SubsectionMetaArg,
    SubsectionMetaVal,
    ABCData(bool),
    Annotation
}
#[derive(Debug)]
enum LSTParsingState {
    MarginStart,
    LineStart,
    SongName,
    MetaArg,
    MetaVal,
    MetaEnd,
    MetaInlineAnnotation,
    Annotation,
    InlineDelimOpen,
    InlineSong(u8),
    InlineMetaArg,
    InlineMetaVal,
    InlineMetaEnd,
    EndInline,
    MaybeInlineMeta,
}

#[derive(PartialEq, Eq, Debug)]
enum ParserStatus {
    New,
    Processing,
    Completed,
    Error,
}

const SUBSECTION_DELIM : &str = "---\n";
pub struct SHBParser {
    line: usize,
    byte_offset: usize,
    utf16_offset: usize,
    errors: Vec<SHBParseError>,
    annotation_buffer : String,
    line_has_content: bool,
    meta_arg_buffer: String,
    meta_val_buffer: String,
    section_id_buffer: String,
    section_desc_buffer: String,
    section_meta_arg_buffer: String,
    section_meta_val_buffer: String,
    lyric_buffer: String,
    chord_buffer: String,
    abc_buffer: String,
    abc_delim_pos : usize,
    line_buffer: Vec<Vec<(Vec<MusicEvent>, Vec<LyricEvent>)>>,
    order_buffer: Vec<(String, Vec<String>)>,
    last_line_buffer: String,
    state: SHBParsingState,
    status: ParserStatus,
    song: Song,
}
pub struct LSTParser {
    line: usize,
    state: LSTParsingState,
    errors: Vec<LSTParseError>,
    byte_offset: usize,
    utf16_offset: usize,
    annotation_buffer : String,
    song_handle_buffer: String,
    meta_arg_buffer: String,
    meta_val_buffer: String,
    last_line_buffer: String,
    inline_song_buffer: String,
    pub content: Songlist,
}
impl Default for SHBParser {
    fn default() -> Self {
        SHBParser {
            line: 1,
            byte_offset: 0,
            utf16_offset: 0,
            errors: Vec::new(),
            line_has_content: false,
            annotation_buffer : String::new(),
            meta_arg_buffer: String::new(),
            meta_val_buffer: String::new(),
            section_id_buffer: String::new(),
            section_desc_buffer: String::new(),
            section_meta_val_buffer: String::new(),
            section_meta_arg_buffer: String::new(),
            abc_buffer: String::new(),
            abc_delim_pos : 0,
            last_line_buffer: String::new(),
            line_buffer: Vec::new(),
            order_buffer: Vec::new(),
            lyric_buffer: String::new(),
            chord_buffer: String::new(),
            state: SHBParsingState::MetaStart,
            status: ParserStatus::New,
            song: Song::default(),
        }
    }
}
impl Default for LSTParser {
    fn default() -> Self {
        LSTParser {
            line: 1,
            state: LSTParsingState::MarginStart,
            errors: Vec::new(),
            byte_offset: 0,
            utf16_offset: 0,
            annotation_buffer: String::new(),
            inline_song_buffer: String::new(),
            song_handle_buffer: String::new(),
            meta_arg_buffer: String::new(),
            meta_val_buffer: String::new(),
            last_line_buffer: String::new(),
            content: Vec::new(),
        }
    }
}
//Just to keep consistency
macro_rules! consume_cls {
    ($sel:expr) => {{
        $sel.clear();
    }};
}

macro_rules! consume {
    ($sel:expr) => {{
        //Check if take is faster
        let u = $sel.clone();
        $sel.clear();
        u
    }};
}

macro_rules! consume_str {
    ($sel:expr,$s:expr) => {{
        let u = $s.to_owned();
        $sel.clear();
        u
    }};
}

macro_rules! last_section {
    ($sel:expr) => {
        $sel.song.sections.last_mut().unwrap()
    };
}

macro_rules! last_subsection {
    ($sel:expr) => {
        if let SongBlock::Section(section) = last_section!($sel){
            section.subsections.last_mut().unwrap()
        }else{
            unreachable!()    
        }
    };
}

macro_rules! buffer_measure {
    ($sel:expr) => {
        $sel.line_buffer.last_mut().unwrap()
    };
}

macro_rules! buffer_block {
    ($sel:expr) => {
        buffer_measure!($sel).last_mut().unwrap()
    };
}

macro_rules! push_error {
    ($self:expr,$start_off:expr,$end_off:expr,$data:expr) => {
        $self.errors.push(LocatedError {
            loc: ($self.byte_offset - $start_off..$self.byte_offset - $end_off),
            //TODO: FIX utf16 offset
            loc16: ($self.utf16_offset - $start_off..$self.utf16_offset-$end_off),
            line: $self.line,
            kind: $data,
            context: None,
        })
    };
}

macro_rules! unexpected_char {
    ($self:expr,$c:expr) => {
        push_error!($self, 1, 0, SHBErrorKind::UnexpectedChar($c))
    };
}

macro_rules! parse_lyric_buffer {
    ($self:expr) => {{
        let mut fragment_start: usize = 0;
        for (i, c) in $self.lyric_buffer.char_indices() {
            if c == '^' {
                if i > fragment_start {
                    let slice = &$self.lyric_buffer[fragment_start..i];
                    buffer_block!($self)
                        .1
                        .push(LyricEvent::LyricText(slice.to_owned()));
                    //Caret has length 1
                    fragment_start = i + 1;
                }
                buffer_block!($self).1.push(LyricEvent::LyricBreak);
            }
        }
        if fragment_start < $self.lyric_buffer.len() {
            buffer_block!($self)
                .1
                .push(LyricEvent::LyricText(consume_str!(
                    $self.lyric_buffer,
                    $self.lyric_buffer[fragment_start..]
                )));
        }
    }};
}

macro_rules! set_if_entry {
    ($sel:expr,$member:ident,$val:expr) => {{
        let last_elem = $sel.content.last_mut();
        if let Some(SonglistElement::Entry(ref mut entry)) = last_elem{
            entry.$member = $val;   
        }
    }};
}

impl SHBParser {
    pub fn clear(&mut self){
        self.line = 1;
        self.byte_offset = 0;
        self.line_has_content = false;
        self.annotation_buffer.clear();
        self.meta_arg_buffer.clear();
        self.meta_val_buffer.clear();
        self.section_id_buffer.clear();
        self.section_desc_buffer.clear();
        self.section_meta_arg_buffer.clear();
        self.section_meta_val_buffer.clear();
        self.last_line_buffer.clear();
        self.line_buffer.clear();
        self.line_buffer.clear();
        self.order_buffer.clear();
        self.lyric_buffer.clear();
        self.chord_buffer.clear();
        self.abc_buffer.clear();
        self.state = SHBParsingState::MetaStart;
        self.status = ParserStatus::New;
    }
    pub fn parse_char(&mut self, c: char) {
        if let ParserStatus::New = self.status {
            self.status = ParserStatus::Processing;
        }
        use SHBParsingState::*;
        let mut retry = true;

        if c == '\r' {
            self.byte_offset += 1;
            self.utf16_offset += 1;
            return; //Do nothing
        }
        self.last_line_buffer.push(c);

        while retry {
            retry = false;
            //println!("Line:{} Char:{:?} State:{:?}",self.line,c,self.state);
            match self.state {
                MetaStart => match c {
                    _ if c.is_whitespace() => {
                        //Do nothing
                    }
                    '@' => {
                        self.state = SectionStart;
                    }
                    '#' => {
                        self.state = Annotation;
                    }
                    '_' => {
                        self.state = MetaArg;
                        retry = true;
                        continue;
                    }
                    _ if c.is_alphanumeric() => {
                        self.state = MetaArg;
                        retry = true;
                        continue;
                    }
                    _ => {
                        unexpected_char!(self,c);
                        //eprintln!("Unexpected {}", c);
                    }
                },
                MetaArg => match c {
                    ':' => {
                        self.state = MetaVal;
                    }
                    '\n' => {
                        //eprintln!("Metadata '{}' without assigned value",self.meta_arg_buffer);
                        push_error!(
                            self,
                            self.meta_arg_buffer.len(),
                            0,
                            SHBErrorKind::NoMetaValue(self.meta_arg_buffer.trim().to_string())
                        );
                        self.meta_arg_buffer.clear();
                        self.state = MetaStart;
                    }
                    ' ' | '_' => {
                        self.meta_arg_buffer.push(c);
                    }
                    _ if c.is_alphanumeric() => {
                        self.meta_arg_buffer.push(c);
                    }
                    _ => {
                        unexpected_char!(self, c);
                        //eprintln!("Unexpected character {}",c)
                    }
                },
                Annotation => match c{
                    '\n' => {
                        
                        if let Some(SongBlock::Annotation(s)) = self.song.sections.last_mut(){
                            s.push_str(&self.annotation_buffer);
                           self.annotation_buffer.clear();
                        }else{
                            self.annotation_buffer.push('\n');
                            self.song.sections.push(SongBlock::Annotation(self.annotation_buffer.clone()));
                            self.annotation_buffer.clear();
                        }
                        self.state = SectionOrAnnotationStart;
                        
                    }
                    _ => {
                        self.annotation_buffer.push(c);
                    }
                }
                MetaVal => match c {
                    '\n' => {
                        let trim_arg = self.meta_arg_buffer.trim();
                        if trim_arg.is_empty() {
                            eprintln!("Empty metadata name");
                        }
                        let trim_val = self.meta_val_buffer.trim();
                        if trim_val.is_empty() {
                            eprintln!("Empty metadata value");
                        }
                        match trim_arg {
                            "name" => {
                                self.song.name = consume_str!(self.meta_val_buffer, trim_val);
                                consume_cls!(self.meta_arg_buffer);
                            }
                            "tonic" => {
                                if let Some(tonality) = parse_tonality(trim_val) {
                                    self.song.tonality = tonality
                                } else {
                                    push_error!(
                                        self,
                                        self.meta_val_buffer.len(),
                                        0,
                                        SHBErrorKind::WrongTonicFormat
                                    );
                                }
                                consume_cls!(self.meta_arg_buffer);
                                consume_cls!(self.meta_val_buffer);
                            }
                            "order" | "version" => {
                                let parts = self
                                    .meta_val_buffer
                                    .split_whitespace()
                                    .collect::<Vec<&str>>();
                                if parts.len() < 2 {
                                    push_error!(
                                        self,
                                        self.meta_val_buffer.len(),
                                        0,
                                        SHBErrorKind::WrongOrderFormat(trim_val.to_string())
                                    )
                                } else {
                                    self.order_buffer.push((
                                        parts[0].to_owned(),
                                        parts[1..].iter().map(|x| x.to_string()).collect(),
                                    ));
                                }
                            }
                            "bpm" => {
                                if let Ok(bpm) = trim_val.parse::<f32>() {
                                    self.song.bpm = Some(bpm);
                                }
                                consume_cls!(self.meta_arg_buffer);
                                consume_cls!(self.meta_val_buffer);
                            }
                            _ => {
                                self.song.metadata.insert(
                                    consume_str!(self.meta_arg_buffer, trim_arg),
                                    consume_str!(self.meta_val_buffer, trim_val),
                                );
                            }
                        }
                        self.state = MetaStart;
                    }

                    _ => {
                        self.meta_val_buffer.push(c);
                    }
                },

                SectionStart => {
                    self.song.sections.push(SongBlock::default());
                    self.state = SectionHeadId;
                    retry = true;
                    continue;
                }

                SectionHeadId => match c {
                    '\n' => {
                        let id_trim = self.section_id_buffer.trim();
                        if id_trim.is_empty() {
                            push_error!(
                                self,
                                self.section_id_buffer.len(),
                                1,
                                SHBErrorKind::MissingSectionID
                            );
                            //eprintln!("Empty section name")
                        }
                        self.song
                            .section_names
                            .insert(id_trim.to_owned(), self.song.sections.len() - 1);
                        last_section!(self).section_unwrap_mut().name = consume_str!(self.section_id_buffer, id_trim);
                        self.state = SubsectionStart;
                    }
                    _ if c.is_whitespace() => {
                        let id_trim = self.section_id_buffer.trim();
                        if id_trim.is_empty() {
                            //eprintln!("Empty section name")
                            push_error!(
                                self,
                                self.section_id_buffer.len(),
                                1,
                                SHBErrorKind::MissingSectionID
                            );
                        }
                        last_section!(self).section_unwrap_mut().name = consume_str!(self.section_id_buffer, id_trim);
                        self.state = SectionHeadDesc;
                    }
                    _ if c.is_alphanumeric() => {
                        self.section_id_buffer.push(c);
                    }
                    '~' => {
                        self.section_id_buffer.push(c);
                    }
                    _ => {
                        unexpected_char!(self, c);
                        //eprintln!("unexpected {}",c);
                    }
                },

                SectionHeadDesc => match c {
                    '\n' | '|' => {
                        let desc_trim = self.section_desc_buffer.trim();
                        self.song.section_names.insert(
                            last_section!(self).section_unwrap_mut().name.clone(),
                            self.song.sections.len() - 1,
                        );
                        last_section!(self).section_unwrap_mut().description =
                            consume_str!(self.section_desc_buffer, desc_trim);
                        self.state = if c == '\n' {
                            SubsectionStart
                        } else {
                            SectionMetaArg
                        }
                    }

                    _ => self.section_desc_buffer.push(c),
                },

                SectionMetaArg => match c {
                    '\n' => {
                        //No zero value arguments
                        push_error!(
                            self,
                            self.section_meta_arg_buffer.len(),
                            0,
                            SHBErrorKind::NoMetaValue(
                                self.section_meta_arg_buffer.trim().to_string()
                            )
                        );
                        consume_cls!(self.section_meta_arg_buffer);
                        self.state = SubsectionStart;
                    }
                    ':' => {
                        self.state = SectionMetaVal;
                    }
                    _ => {
                        self.section_meta_arg_buffer.push(c);
                    }
                },

                SectionMetaVal => match c {
                    c @ ('\n' | '|') => {
                        let trim_arg = self.section_meta_arg_buffer.trim();
                        let trim_val = self.section_meta_val_buffer.trim();
                        match trim_arg {
                            "tonic" | "delta" => {
                                if let Some((delta, "")) = parse_int_until(trim_val) {
                                    let delta = (((delta % 12) + 12) % 12) as u8;
                                    last_section!(self).section_unwrap_mut().local_tonality = Some(Tonality {
                                        tonic: (self.song.tonality.tonic + delta) % 12,
                                        kind: self.song.tonality.kind,
                                    });
                                } else if let Some(tonality) = parse_tonality(trim_val) {
                                    last_section!(self).section_unwrap_mut().local_tonality = Some(tonality);
                                } else {
                                    push_error!(
                                        self,
                                        self.section_meta_val_buffer.len(),
                                        0,
                                        SHBErrorKind::WrongTonicFormat
                                    )
                                }
                            }
                            _ => {
                                push_error!(
                                    self,
                                    self.section_meta_arg_buffer.len()
                                        + self.section_meta_val_buffer.len(),
                                    self.section_meta_val_buffer.len(),
                                    SHBErrorKind::UnknownMetaArgument(
                                        self.section_meta_val_buffer.clone()
                                    )
                                )
                            }
                        }
                        consume_cls!(self.section_meta_arg_buffer);
                        consume_cls!(self.section_meta_val_buffer);
                        if c == '\n' {
                            self.state = SubsectionStart;
                        } else {
                            self.state = SectionMetaArg;
                        }
                    }

                    _ => {
                        self.section_meta_val_buffer.push(c);
                    }
                },

                SubsectionStart => {
                    last_section!(self).section_unwrap_mut().subsections.push(Subsection::default());
                    self.state = LineStart;
                    retry = true;
                    continue;
                }
                SectionOrAnnotationStart => match c{
                    '@' => {
                        self.state = SectionStart;
                    }
                    ' ' | '\n' => {
                    }
                    '#' => {
                        self.state = Annotation;
                    }
                    _ => {
                        unexpected_char!(self,c);
                    }
                }
                LineStart => match c {
                    '@' => {
                        self.state = SectionStart;
                        self.lyric_buffer.clear();
                    }
                    '#'=>{
                        self.lyric_buffer.clear();
                        self.state = Annotation;
                    }
                    '\n' =>{
                        self.lyric_buffer.clear();
                    }
                    ' ' => {
                        self.lyric_buffer.push(' ');
                    }
                    '-' => {
                        self.lyric_buffer.push('-');
                        self.state = SubsectionDelim(1);
                    }
                    _ => {
                        self.state = MeasureStart;
                        retry = true;
                        continue;
                    }
                },

                MeasureStart => {
                    self.state = BlockStart;
                    self.line_buffer.push(Vec::new());
                    retry = true;
                    continue;
                }

                BlockStart => {
                    self.state = MaybeLyricBlock(false);
                    buffer_measure!(self).push((Vec::new(), Vec::new()));
                    retry = true;
                    continue;
                }

                MaybeLyricBlock(maybe_lit) => match c {
                    '\n' => {
                        //Parse delayed string
                        parse_lyric_buffer!(self);

                        self.state = LineEnd;
                        retry = true;
                        continue;
                    }
                    '·' | '*' if !maybe_lit => {
                        std::mem::swap(&mut self.lyric_buffer, &mut self.chord_buffer);
                        let (events, remainder, mut c_errors) =
                            parse_music_block(&self.chord_buffer);
                        let err_offset = self.byte_offset - self.chord_buffer.len();
                        for err in &mut c_errors {
                            err.loc.start += err_offset;
                            err.loc.end += err_offset;
                            err.line = self.line;
                        }
                        self.errors.append(&mut c_errors);
                        if !remainder.is_empty() {
                            push_error!(
                                self,
                                self.chord_buffer.len(),
                                0,
                                SHBErrorKind::MalformedMusicEvent(remainder.to_owned())
                            );
                        }
                        buffer_block!(self).0 = events;
                        consume_cls!(self.chord_buffer);
                        self.state = TrueLyricBlock(c);
                    }
                    '"' => {
                        self.lyric_buffer.push(c);
                        self.state = MaybeLyricBlock(!maybe_lit);
                    }
                    '|' => {
                        parse_lyric_buffer!(self);
                        self.state = MeasureStart;
                    }
                    '´' | '`' => {
                        parse_lyric_buffer!(self);
                        self.state = BlockStart;
                    }
                    _ => {
                        self.lyric_buffer.push(c);
                    }
                },

                TrueLyricBlock(delim) => match c {
                    '|' => {
                        buffer_block!(self)
                            .1
                            .push(LyricEvent::LyricText(consume!(self.lyric_buffer)));
                        self.state = MeasureStart;
                    }
                    '´' | '`' => {
                        buffer_block!(self)
                            .1
                            .push(LyricEvent::LyricText(consume!(self.lyric_buffer)));
                        self.state = BlockStart;
                    }
                    '·' | '*' if c == delim => {
                        push_error!(self, 1, 0, SHBErrorKind::RepeatedDot);
                    }
                    '^' => {
                        buffer_block!(self)
                            .1
                            .push(LyricEvent::LyricText(consume!(self.lyric_buffer)));
                        buffer_block!(self).1.push(LyricEvent::LyricBreak);
                    }
                    '\n' => {
                        buffer_block!(self)
                            .1
                            .push(LyricEvent::LyricText(consume!(self.lyric_buffer)));

                        self.state = LineEnd;
                        retry = true;
                        continue;
                    }
                    _ => {
                        self.lyric_buffer.push(c);
                    }
                },

                LineEnd => {
                    self.state = LineStart;
                    if !self.line_has_content {
                        self.line_buffer.clear();
                        break;
                    }
                    let mut has_chords = false;
                    let mut has_lyrics = false;
                    for (chord_block, lyric_block) in self.line_buffer.iter().flatten() {
                        for lyric_fragment in lyric_block {
                            if let LyricEvent::LyricText(text) = lyric_fragment {
                                if !text.trim().is_empty() {
                                    has_lyrics = true;
                                }
                            }
                        }
                        if !chord_block.is_empty() {
                            has_chords = true;
                        }
                    }
                    match (has_chords, has_lyrics) {
                        (true, true) => {
                            last_subsection!(self)
                                .lines
                                .push(Line::Mixed(consume!(self.line_buffer)));
                        }
                        (true, false) => {
                            let u = self
                                .line_buffer
                                .iter()
                                .map(|measure| {
                                    measure.iter().map(|(chords, _)| chords.clone()).collect()
                                })
                                .collect();

                            last_subsection!(self).lines.push(Line::Chords(u))
                        }
                        (false, _) => {
                            let u = self
                                .line_buffer
                                .iter()
                                .map(|measure| {
                                    measure.iter().map(|(_, lyrics)| lyrics.clone()).collect()
                                })
                                .collect();

                            last_subsection!(self).lines.push(Line::Lyrics(u))
                        }
                    }
                    self.line_buffer.clear();
                }
                SubsectionDelim(1) => match c {
                    '-' => {
                        self.state = SubsectionDelim(2);
                        self.lyric_buffer.push('-');
                    }
                    _ => {
                        self.state = MeasureStart;
                        retry = true;
                        continue;
                    }
                },
                SubsectionDelim(2) => match c {
                    '-' => {
                        self.state = SubsectionDelim(3);
                        self.lyric_buffer.clear();
                    }
                    '!' => {
                        self.state = SubsectionMetaArg;
                        self.lyric_buffer.clear();
                    }
                    '>' => {
                        self.state = ABCData(false);
                        self.abc_delim_pos = 0;
                        self.lyric_buffer.clear();
                    }
                    _ => {
                        self.state = MeasureStart;
                        retry = true;
                        continue;
                    }
                },
                SubsectionDelim(3) => match c {
                    '\n' => {
                        self.state = SubsectionStart;
                    }
                    '-' => {}
                    _ if c.is_whitespace() => {}
                    _ => {
                        unexpected_char!(self, c);
                        //eprint!("Unexpected {}",c)
                    }
                },
                ABCData(line_start) => match c {
                    c if self.abc_delim_pos > 0 && (&SUBSECTION_DELIM).chars().nth(self.abc_delim_pos) == Some(c) =>{
                        self.abc_delim_pos += 1;
                        if self.abc_delim_pos >= SUBSECTION_DELIM.len() {
                            last_section!(self).section_unwrap_mut().has_abc = true;
                            last_subsection!(self).abc = Some(consume!(self.abc_buffer));
                            self.state = SubsectionStart;
                        }else{
                            self.state = ABCData(false);
                        }
                    }
                    c if self.abc_delim_pos > 0 => {
                        self.abc_buffer += &SUBSECTION_DELIM[0..self.abc_delim_pos];
                        self.abc_buffer.push(c);
                        self.abc_delim_pos = 0;
                        if c == '\n' {
                            self.state = ABCData(true);
                        }else{
                            self.state = ABCData(false);
                        }
                    }
                    '\n' =>{
                        self.state = ABCData(true);
                        self.abc_buffer.push(c);
                    }
                    '@' if line_start => {
                        last_section!(self).section_unwrap_mut().has_abc = true;
                        last_subsection!(self).abc = Some(consume!(self.abc_buffer));
                        self.state = SectionStart;
                    }
                    '-' if line_start => {
                        self.state = ABCData(false);
                        self.abc_delim_pos = 1;
                    }
                    _ =>{
                        self.state = ABCData(false);
                        self.abc_buffer.push(c);
                    }
                }
                SubsectionMetaArg => match c {
                    ':' => {
                        self.state = SubsectionMetaVal;
                    }
                    ' ' | '_' => {
                        self.meta_arg_buffer.push(c);
                    }
                    '\n' => {
                        eprintln!(
                            "Subsection metadata '{}' without assigned value",
                            self.meta_arg_buffer
                        );
                        self.meta_arg_buffer.clear();
                        self.state = LineStart;
                    }
                    _ if c.is_alphanumeric() => {
                        self.meta_arg_buffer.push(c);
                    }
                    _ => {
                        unexpected_char!(self, c);
                        //eprintln!("Unexpected character {}",c)
                    }
                },
                SubsectionMetaVal => match c {
                    '\n' => {
                        let trim_arg = self.meta_arg_buffer.trim();
                        if trim_arg.is_empty() {
                            eprintln!("Empty metadata name");
                            push_error!(self, 0, 0, SHBErrorKind::NoMetaName)
                        }
                        let trim_val = self.meta_val_buffer.trim();
                        if trim_val.is_empty() {
                            eprintln!("Empty metadata value");
                            push_error!(
                                self,
                                0,
                                0,
                                SHBErrorKind::NoMetaValue(trim_arg.to_string())
                            );
                        }
                        let mut arg = consume_str!(self.meta_arg_buffer, trim_arg);
                        arg.make_ascii_lowercase();
                        last_subsection!(self)
                            .metadata
                            .insert(arg, consume_str!(self.meta_val_buffer, trim_val));
                        self.state = LineStart;
                    }

                    _ => {
                        self.meta_val_buffer.push(c);
                    }
                },
                _ => {}
            }
        }
        if c == '\n' {
            self.line += 1;
            self.line_has_content = false;
            self.errors
                .iter_mut()
                .rev()
                .take_while(|e| e.context.is_none())
                .for_each(|e| {
                    e.context =
                        Some(self.last_line_buffer[..self.last_line_buffer.len() - 1].to_string())
                });
            self.last_line_buffer.clear();
        } else if !c.is_whitespace() {
            self.line_has_content = true;
        }
        self.byte_offset += c.len_utf8();
        self.utf16_offset += c.len_utf16();
    }
    pub fn parse_str(&mut self, s: &str) {
        for c in s.chars() {
            self.parse_char(c);
        }
        self.finalize();
    }
    fn process_order(&mut self) {
        for (name, sections) in self.order_buffer.drain(..) {
            let indexes = sections
                .iter()
                .filter_map(|sname| {
                    let idx = self.song.section_names.get(sname).copied();
                    if idx.is_none() {
                        push_error!(
                            self,
                            self.meta_val_buffer.len(),
                            0,
                            SHBErrorKind::SectionNotFound(sname.to_string())
                        )
                    }
                    idx
                })
                .collect::<Vec<usize>>();

            if self.song.orders.insert(name.clone(), indexes).is_some() {
                push_error!(
                    self,
                    self.meta_val_buffer.len(),
                    0,
                    SHBErrorKind::RepeatedOrderName(name)
                )
            }
        }
    }
    pub fn get_total_lines(&self) -> usize {
        self.line
    }
    pub fn finalize(&mut self) {
        self.parse_char('\n');
        if self.state == SHBParsingState::ABCData(true){
            last_section!(self).section_unwrap_mut().has_abc = true;
            last_subsection!(self).abc = Some(consume!(self.abc_buffer));
        }
        self.process_order();
        if self.status != ParserStatus::Error {
            self.status = ParserStatus::Completed;
        }
        self.line -= 1;
    }
    pub fn extract(&mut self) -> (Song, Vec<SHBParseError>) {
        (std::mem::take(&mut self.song), std::mem::take(&mut self.errors))
    }
    pub fn extract_song(&mut self) -> Song {
        std::mem::take(&mut self.song)
    }
}


impl LSTParser {
    pub fn clear(&mut self) {
        self.line = 1;
        self.state = LSTParsingState::MarginStart;
        self.byte_offset = 0;
        self.utf16_offset = 0;
        self.annotation_buffer.clear();
        self.inline_song_buffer.clear();
        self.song_handle_buffer.clear();
        self.meta_arg_buffer.clear();
        self.meta_val_buffer.clear();
        self.last_line_buffer.clear();
    }
    pub fn parse_char(&mut self, c: char) {
        let mut retry = true;
        if c == '\r' {
            self.byte_offset += 1;
            self.utf16_offset += 1;
            return;
        }
        self.last_line_buffer.push(c);
        while retry {
            //println!("{:?} {:?}",c,self.state);
            retry = false;
            match self.state {
                LSTParsingState::MarginStart => match c {
                    '<' => {
                            self.state = LSTParsingState::InlineDelimOpen;
                            self.song_handle_buffer.push('<');
                        }
                    '#' => {
                        self.state = LSTParsingState::Annotation;
                    }
                    '|' | '/' => {
                        let last_entry = self.content.last_mut();
                        if matches!(last_entry,Some(SonglistElement::Entry(_))){
                            self.state = LSTParsingState::MetaArg;
                        }else{
                            retry = true;
                            self.state = LSTParsingState::LineStart;
                        }
                    }
                    _ if !c.is_whitespace() => {
                            retry = true;
                            self.state = LSTParsingState::LineStart;
                    }
                    _ => {}
                }
                LSTParsingState::Annotation => match c {
                    '\n'=>{
                        self.content.push(SonglistElement::Annotation(self.annotation_buffer.clone()));
                        self.annotation_buffer.clear();
                        self.state = LSTParsingState::MarginStart;
                    }
                    _ =>{
                        self.annotation_buffer.push(c);
                    }
                }
                LSTParsingState::LineStart => {
                    self.content.push(SonglistElement::Entry(SonglistEntry {
                        id_file: self.song_handle_buffer.trim().to_owned(),
                        rename: None,
                        tonic: None,
                        explicit_order: None,
                        named_order: None,
                        joined: false,
                        inline_data: None,
                        line: self.line,
                        utf16_offset: self.utf16_offset
                    }));
                    retry = true;
                    self.state = LSTParsingState::SongName;
                }
                LSTParsingState::MetaInlineAnnotation =>{
                    //TODO:
                    unimplemented!();
                }
                LSTParsingState::SongName => match c {
                    '\n' => {
                        retry = true;
                        self.state = LSTParsingState::MarginStart;
                        let trim_name = self.song_handle_buffer.trim();
                        if trim_name.is_empty() {
                            push_error!(
                                self,
                                self.song_handle_buffer.len(),
                                0,
                                LSTErrorKind::MissingSongName
                            )
                        } else {
                            set_if_entry!(self,id_file,trim_name.to_owned());
                            //self.content.last_mut().unwrap().id_file = trim_name.to_owned();
                        }
                        self.song_handle_buffer.clear();
                    }
                    '|' | '/' => {
                        self.state = LSTParsingState::MetaArg;
                        let trim_name = self.song_handle_buffer.trim();
                        if trim_name.is_empty() {
                            push_error!(
                                self,
                                self.song_handle_buffer.len(),
                                0,
                                LSTErrorKind::MissingSongName
                            )
                        } else {
                            set_if_entry!(self,id_file,trim_name.to_owned());
                            //self.content.last_mut().unwrap().id_file = trim_name.to_owned();
                        }
                        self.song_handle_buffer.clear();
                    }
                    _ => {
                        self.song_handle_buffer.push(c);
                    }
                },
                LSTParsingState::MetaArg => match c {
                    '|' | '/' | '\n' => {
                        retry = true;
                        self.state = LSTParsingState::MetaEnd;
                    }
                    ':' => {
                        self.state = LSTParsingState::MetaVal;
                    }
                    _ => {
                        self.meta_arg_buffer.push(c);
                    }
                },
                LSTParsingState::MetaVal => match c {
                    '|' | '/' |'\n' => {
                        retry = true;
                        self.state = LSTParsingState::MetaEnd;
                    }
                    _ => {
                        self.meta_val_buffer.push(c);
                    }
                },
                LSTParsingState::MetaEnd => {
                    self.meta_arg_buffer.make_ascii_lowercase();
                    let trim_arg = self.meta_arg_buffer.trim();
                    let trim_val = self.meta_val_buffer.trim();
                    if trim_arg.is_empty() {
                        push_error!(
                            self,
                            self.meta_arg_buffer.len(),
                            0,
                            LSTErrorKind::MissingMetaName
                        );
                    } else {
                        match trim_arg {
                            "tonic" => {
                                if let Some(tonality) = parse_tonality(trim_val) {
                                    set_if_entry!(self,tonic,Some(tonality));
                                } else {
                                    push_error!(
                                        self,
                                        self.meta_arg_buffer.len(),
                                        0,
                                        LSTErrorKind::WrongTonicFormat
                                    );
                                }
                            }
                            "rename" | "name" => {
                                set_if_entry!(self,rename,Some(trim_val.to_owned()));
                            }
                            "joined" | "join" => {
                                set_if_entry!(self,joined,true);
                            }
                            "order" => {
                                if let Some(SonglistElement::Entry(entry)) = self.content.last_mut(){
                                    let v = parse_order_list(trim_val);
                                    if let Some(ref mut order) = entry.explicit_order{
                                        order.extend(v);
                                    }else{
                                        entry.explicit_order = Some(
                                            v
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                        self.meta_arg_buffer.clear();
                        self.meta_val_buffer.clear();
                        match c {
                            '\n' => {
                                self.state = LSTParsingState::MarginStart;
                            }
                            '|' | '/' => {
                                self.state = LSTParsingState::MetaArg;
                            }
                            _ => {
                                unreachable!();
                            }
                        }
                    }
                }
                LSTParsingState::InlineDelimOpen => {
                    if c == '<' {
                        self.state = LSTParsingState::MaybeInlineMeta;
                        self.song_handle_buffer.clear();
                        self.content.push(SonglistElement::Entry(SonglistEntry {
                            id_file: String::new(),
                            rename: None,
                            tonic: None,
                            explicit_order: None,
                            named_order: None,
                            joined: false,
                            inline_data: None,
                            line: self.line,
                            utf16_offset: self.utf16_offset
                        }));
                    } else {
                        self.state = LSTParsingState::LineStart;
                    }
                }
                LSTParsingState::MaybeInlineMeta => match c {
                    '\n' => self.state = LSTParsingState::InlineSong(0),
                    _ if c.is_ascii_alphabetic() => {
                        retry = true;
                        self.state = LSTParsingState::InlineMetaArg;
                    }

                    _ if c.is_whitespace() => {}
                    _ => {
                        push_error!(self, 0, 0, LSTErrorKind::UnexpectedChar(c));
                    }
                },

                LSTParsingState::InlineMetaArg => match c {
                    '|' | '/' | '\n' => {
                        retry = true;
                        self.state = LSTParsingState::InlineMetaEnd;
                    }
                    ':' => {
                        self.state = LSTParsingState::InlineMetaVal;
                    }
                    _ => {
                        self.meta_arg_buffer.push(c);
                    }
                },
                LSTParsingState::InlineMetaVal => match c {
                    '|' | '/' | '\n' => {
                        retry = true;
                        self.state = LSTParsingState::InlineMetaEnd;
                    }
                    _ => {
                        self.meta_val_buffer.push(c);
                    }
                },

                LSTParsingState::InlineMetaEnd => {
                    self.meta_arg_buffer.make_ascii_lowercase();
                    let trim_arg = self.meta_arg_buffer.trim();
                    let trim_val = self.meta_val_buffer.trim();
                    if trim_arg.is_empty() {
                        push_error!(
                            self,
                            self.meta_arg_buffer.len(),
                            0,
                            LSTErrorKind::MissingMetaName
                        );
                    } else {
                        match trim_arg {
                            "tonic" => {
                                if let Some(tonality) = parse_tonality(trim_val) {
                                    set_if_entry!(self,tonic,Some(tonality));
                                } else {
                                    push_error!(
                                        self,
                                        self.meta_arg_buffer.len(),
                                        0,
                                        LSTErrorKind::WrongTonicFormat
                                    );
                                }
                            }
                            "rename" | "name" => {
                                set_if_entry!(self,rename,Some(trim_val.to_owned()));
                            }
                            "joined" | "join" => {
                                set_if_entry!(self,joined,true);
                            }
                            "order" => {
                                //TODO:(?)
                                eprintln!("Unimplemented ordering in inline songs")
                            }
                            _ => {}
                        }
                        self.meta_arg_buffer.clear();
                        self.meta_val_buffer.clear();
                        match c {
                            '\n' => {
                                self.state = LSTParsingState::InlineSong(0);
                            }
                            '|' | '/' => {
                                self.state = LSTParsingState::InlineMetaArg;
                            }
                            _ => {
                                unreachable!();
                            }
                        }
                    }
                }
                LSTParsingState::InlineSong(n) => match c {
                    '\n' => {
                        self.state = LSTParsingState::InlineSong(1);
                        self.inline_song_buffer.push(c);
                    }
                    '>' if n == 1 => {
                        self.state = LSTParsingState::InlineSong(n + 1);
                        self.inline_song_buffer.push(c);
                    }
                    '>' if n == 2 => {
                        self.state = LSTParsingState::EndInline;
                        //Parse the buffer
                        let mut parser = SHBParser::default();
                        parser.parse_str(
                            &self.inline_song_buffer[..self.inline_song_buffer.len() - 1],
                        );
                        //println!("{:?}",self.inline_song_buffer);
                        let (res, mut errors) = parser.extract();
                        set_if_entry!(self,inline_data,Some(res));
                        let err_offset = self.byte_offset - self.inline_song_buffer.len();
                        let err_utf16_offset = self.utf16_offset - self.inline_song_buffer.encode_utf16().count();
                        let errors = errors.drain(..).map(|err| LSTParseError {
                            loc: (err_offset + err.loc.start..err_offset + err.loc.end),
                            loc16: (err_utf16_offset + err.loc16.start..err_utf16_offset + err.loc16.end),
                            line: self.line + err.line - 1,
                            kind: LSTErrorKind::InlineSongError(err.kind),
                            context: err.context,
                        });
                        self.errors.extend(errors);
                        self.inline_song_buffer.clear();
                    }
                    ' ' | '\t' => {
                        self.state = LSTParsingState::InlineSong(n);
                        self.inline_song_buffer.push(c);
                    }
                    _ => {
                        self.state = LSTParsingState::InlineSong(0);
                        self.inline_song_buffer.push(c);
                    }
                },
                LSTParsingState::EndInline => match c {
                    '\n' => {
                        self.state = LSTParsingState::MarginStart;
                    }
                    ' ' => {}
                    _ => {
                        push_error!(self, 1, 0, LSTErrorKind::UnexpectedChar(c));
                    }
                },
            }
        }
        if c == '\n' {
            self.line += 1;
            self.errors
                .iter_mut()
                .rev()
                .take_while(|e| e.context.is_none())
                .for_each(|e| {
                    e.context =
                        Some(self.last_line_buffer[..self.last_line_buffer.len() - 1].to_string())
                });
            self.last_line_buffer.clear();
        }
        self.byte_offset += c.len_utf8();
        self.utf16_offset += c.len_utf16();
    }

    pub fn parse_str(&mut self, s: &str) {
        for c in s.chars() {
            self.parse_char(c);
        }
        self.finalize();
    }
    pub fn finalize(&mut self) {
        self.parse_char('\n');
        //if self.status != ParserStatus::Error{
        //    self.status = ParserStatus::Completed;
        //}
    }
    pub fn extract(&mut self) -> (Songlist, Vec<LSTParseError>) {
        (std::mem::take(&mut self.content), std::mem::take(&mut self.errors))
    }
}
pub fn parse_shb(song: &str) -> (Song, Vec<SHBParseError>) {
    let mut parser = SHBParser::default();
    parser.parse_str(song);
    parser.extract()
}

pub fn parse_lst(songlist: &str) -> (Songlist, Vec<LSTParseError>) {
    let mut parser = LSTParser::default();
    parser.parse_str(songlist);
    parser.extract()
}

pub fn parse_tone_root(s: &str) -> Option<(u8, &str)> {
    let mut it = s.chars();
    let first = it.next()?;
    let value: u8 = match first {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    if let Some(second) = it.next() {
        match second {
            '#' => Some(((value + 1) % 12, &s[2..])),
            'b' => Some(((value + 11) % 12, &s[2..])),
            _ => Some((value, &s[1..])),
        }
    } else {
        Some((value, &s[1..]))
    }
}

//This one is ugly, maybe refactor
pub fn parse_chord(s: &str) -> Option<(ChordEvent, &str)> {
    let mut s = s;
    let mut time = None;
    if let Some((ntime, ns)) = parse_time_offset(s) {
        time = Some(ntime);
        s = ns;
    }
    let (root, mut s) = parse_tone_root(s)?;
    let mut kind = ChordKind::Major;
    let mut modifiers: Vec<ChordModifier> = Vec::new();
    let mut bass = None;
    if let Some('m') = s.chars().next() {
        kind = ChordKind::Minor;
        s = &s[1..];
    }
    let mut parens = false;
    loop {
        if let Some((modifier, ns)) = parse_keyword(s) {
            s = ns;
            modifiers.push(modifier)
        } else {
            match s.chars().next() {
                Some('(') if !parens => {
                    parens = true;
                    s = &s[1..];
                }
                Some(')') if parens => {
                    parens = false;
                    s = &s[1..];
                }
                _ => {
                    break;
                }
            }
        }
    }
    match s.chars().next() {
        Some('\'') | Some('/') => {
            if let Some((nbass, ns)) = parse_tone_root(&s[1..]) {
                s = ns;
                bass = Some(nbass);
            }
        }
        _ => {}
    }
    if s.starts_with('?') {
        s = &s[1..]; //Todo: Deprecated notation
    }
    Some((
        ChordEvent {
            root,
            kind,
            modifiers,
            time,
            bass,
        },
        s,
    ))
}
pub fn parse_tonality(s: &str) -> Option<Tonality> {
    let (root, s) = parse_tone_root(s)?;
    let mut kind = TonicKind::Major;
    if s.starts_with('m') {
        kind = TonicKind::Minor;
    }
    Some(
        Tonality{
            tonic : root,
            kind
        }
    )
}
pub fn parse_literal(s: &str) -> Option<(String, &str)> {
    if !s.starts_with('"') {
        return None;
    }
    let s = &s[1..];
    let close_idx = s.find('"')?;
    Some((s[..close_idx].to_owned(), &s[close_idx + 1..]))
}
//Todo: Quote checking
pub fn parse_order_list(s: &str) -> Vec<OrderElement>{
    let chars = s.char_indices();
    let mut out = Vec::new();
    let mut ptr : Option<(usize,usize)> = None;
    enum Mode{
        Whitespace,
        Id,
        Literal
    }
    let mut mode = Mode::Whitespace;
    for (i,c) in chars{
        match mode {
            Mode::Whitespace =>{
                if c == '"'{
                    mode = Mode::Literal;
                }else if !c.is_whitespace(){
                    ptr = Some((i,i+c.len_utf8()));
                    mode = Mode::Id;
                }
            },
            Mode::Id=>{
                if !c.is_whitespace(){
                    ptr.as_mut().unwrap().1 += c.len_utf8();
                    mode = Mode::Id;
                }else{
                    let uptr = ptr.unwrap();
                    out.push(OrderElement::Section(s[uptr.0..uptr.1].to_string()) );
                    mode = Mode::Whitespace;
                    ptr = None;
                }
            },
            Mode::Literal=>{
                if c == '"'{
                    if let Some(uptr) = ptr{
                        out.push(OrderElement::Annotation(s[uptr.0..uptr.1].to_string()));
                        mode = Mode::Whitespace;
                        ptr = None;
                    }else{
                        mode = Mode::Whitespace;
                    }
                }else{
                    if ptr == None{
                        ptr = Some((i,i+c.len_utf8()));
                    }else{
                        ptr.as_mut().unwrap().1 += c.len_utf8();
                    }
                }
            }
        }
    }
    if let Some(uptr) = ptr{
        match mode{
            Mode::Id => {
                out.push(OrderElement::Section(s[uptr.0..uptr.1].to_string()));
            }
            Mode::Literal =>{
                out.push(OrderElement::Annotation(s[uptr.0..uptr.1].to_string()));
            }
            _ => {unreachable!()}
        }
    }
    return out
}

pub fn parse_music_block(s: &str) -> (Vec<MusicEvent>, &str, Vec<SHBParseError>) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut s = s.trim();
    let orig_size = s.len();
    let utf16_size = s.encode_utf16().count();
    while !s.is_empty() {
        if s.starts_with('*') || s.starts_with('·') {
            s = &s[1..];
            break;
        }
        if let Some((evt, ns)) = parse_music_event(s) {
            s = ns.trim_start();
            out.push(evt);
        } else if let Some(i) = s.find(|c: char| c.is_whitespace()) {
            let err_start = orig_size - s.len();
            let err_16_len = s[err_start..err_start+1].encode_utf16().count();
            let err_16_start = utf16_size - s.encode_utf16().count();
            err.push(SHBParseError {
                loc: (err_start..err_start + i),
                loc16: (err_16_start..err_16_start + err_16_len),
                line: 0,
                kind: SHBErrorKind::MalformedMusicEvent(s[..i].to_owned()),
                context: None,
            });
            s = s[i..].trim_start();
        } else {
            break;
        }
    }
    (out, s, err)
}
pub fn parse_music_event(s: &str) -> Option<(MusicEvent, &str)> {
    if let Some((chord, ns)) = parse_chord(s) {
        return Some((MusicEvent::ChordEvent(chord), ns));
    }
    if let Some((melody, ns)) = parse_simple_melody(s) {
        return Some((MusicEvent::MelodyEvent(melody), ns));
    }
    if let Some((i, ns)) = parse_uint_until(s) {
        return Some((MusicEvent::NumberedMeasure(i as u16), ns));
    }
    if let Some((lit, ns)) = parse_literal(s) {
        return Some((MusicEvent::Annotation(lit), ns));
    }
    if let Some(u) = seek_cascade!(s,
        ":-" => MusicEvent::StartRepeat,
        "-:" => MusicEvent::EndRepeat,
        "(" => MusicEvent::OpenParen,
        ")" => MusicEvent::CloseParen,
        "%" =>MusicEvent::RepeatMeasure
    ) {
        return Some(u);
    }
    None
}

pub fn parse_simple_melody(s: &str) -> Option<(Vec<u8>, &str)> {
    let mut s = s;
    let mut melody = Vec::<NoteHeight>::new();
    if !s.starts_with('[') {
        return None;
    }
    s = &s[1..];
    while !s.is_empty() {
        if s.starts_with(' ') {
            s = &s[1..];
            continue;
        }
        if s.starts_with(']') {
            s = &s[1..];
            return Some((melody, s));
        }
        if let Some((root, ns)) = parse_tone_root(s) {
            s = ns;
            melody.push(root);
        } else {
            break;
        }
    }
    None
}

pub fn parse_time_offset(s: &str) -> Option<(TimeOffset, &str)> {
    let mut s = s;
    let first_char = s.chars().next()?;
    let mut neg: i8 = 1;
    match first_char {
        '<' => {
            return Some((
                TimeOffset {
                    beat: -1,
                    num: 1,
                    den: 2,
                },
                &s[1..],
            ));
        }
        '-' => {
            neg = -1;
            s = &s[1..];
        }
        _ => {}
    }
    let (beat, s) = parse_uint_until(s)?;
    let beat = ((beat % 128) as i8) * neg;

    if let Some('+') = s.chars().next() {
        if let Some((num, ns)) = parse_uint_until(&s[1..]) {
            if let Some(',') = ns.chars().next() {
                if let Some((den, s)) = parse_uint_until(&ns[1..]) {
                    return Some((
                        TimeOffset {
                            beat,
                            num: num as u8,
                            den: den as u8,
                        },
                        s,
                    ));
                }
            }
        }
    } else if let Some('\'') = s.chars().next() {
        let s = &s[1..];
        return Some((
            TimeOffset {
                beat,
                num: 1,
                den: 2,
            },
            s,
        ));
    }

    Some((
        TimeOffset {
            beat,
            num: 0,
            den: 1,
        },
        s,
    ))
}
macro_rules! seek_cascade {

    ($s:expr, $key:expr => $value:expr ) => {
        {
            seek($s,$key).map(|ns|($value,ns))
        }
    };

    ($s:expr, $key:expr => $value:expr, $($k:expr => $v:expr),+ ) => {
        {
            if let Some(ns) = seek($s,$key){
                Some(($value,ns))
            }else{
                seek_cascade!($s,$($k => $v),+)
            }
        }
    };
}

pub fn parse_keyword(s: &str) -> Option<(ChordModifier, &str)> {
    use ChordKeyword::*;
    use ChordModifier::*;

    //This is understandable and simple, but expensive
    if let Some(u) = seek_cascade!(s,
        "add11" => Keyword(Add11),
        "add2" => Keyword(Add2),
        "add4" => Keyword(Add4),
        "add9" => Keyword(Add9),
        "sus2" => Keyword(Sus2),
        "sus4" => Keyword(Sus4),
        "Maj" => Keyword(Maj),
        "6/9" => Keyword(K69),
        "aug" => Keyword(Aug),
        "dim" => Keyword(Dim),
        "sus" => Keyword(Sus),
        "11" => Keyword(K11),
        "13" => Keyword(K13),
        "9" => Keyword(K9), //woof
        "7" => Keyword(K7),
        "6" => Keyword(K6),
        "5" => Keyword(K5),
        "M" => Keyword(Maj), //Todo, maybe distinguish shortened versions
        "+" => Keyword(Aug),
        "°" => Keyword(Dim)
    ) {
        return Some(u);
    }

    if let Some((kind, ns)) = seek_cascade!(s,
        "no" => ChordAlterationKind::No,
        "b" => ChordAlterationKind::Flat,
        "#" => ChordAlterationKind::Sharp
    ) {
        let may_digits = ns
            .get(..2)
            .and_then(|num_str| num_str.parse::<u8>().ok())
            .or_else(|| ns.get(..1).and_then(|num_str| num_str.parse::<u8>().ok()));

        if let Some(degree) = may_digits {
            let delta = degree / 10 + 1;
            return Some((
                Alteration(ChordAlteration { kind, degree }),
                &ns[delta as usize..],
            ));
        }
    }
    None
}

fn seek<'i>(s: &'i str, pattern: &str) -> Option<&'i str> {
    let subs = s.get(..pattern.len())?;
    if subs != pattern {
        None
    } else {
        Some(&s[pattern.len()..])
    }
}

pub fn parse_uint_until(s: &str) -> Option<(u32, &str)> {
    if !s.chars().next()?.is_ascii_digit() {
        return None;
    }
    let (i, val) = s
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_digit())
        .fold((0, 0), |(_idx, val), (i, c)| {
            (i, val * 10 + (c as u32 - '0' as u32))
        });
    Some((val, &s[i + 1..]))
}

pub fn parse_int_until(mut s: &str) -> Option<(i32, &str)> {
    let mut neg = false;
    let mut it = s.chars();
    match it.next()? {
        '+' => {
            s = &s[1..];
            let next = it.next()?;
            if !next.is_ascii_digit() {
                return None;
            }
        }
        '-' => {
            neg = true;
            s = &s[1..];
            let next = it.next()?;
            if !next.is_ascii_digit() {
                return None;
            }
        }
        c if !c.is_ascii_digit() => {
            return None;
        }
        _ => {}
    }
    let (i, mut val) = s
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_digit())
        .fold((0, 0), |(_idx, val), (i, c)| {
            (i, val * 10 + (c as u32 - '0' as u32) as i32)
        });
    if neg {
        val = -val;
    }
    Some((val, &s[i + 1..]))
}

pub fn parse_shb_str(s: &str) -> (Song, Vec<SHBParseError>) {
    let mut parser = SHBParser::default();
    parser.parse_str(s);
    parser.extract()
}
