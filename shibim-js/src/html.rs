use markup;
use shibim_parse as base;
use crate::strings;
markup::define!(
    Songlist<'i>(list : &'i Vec<base::CompiledSong>){
        div . "u-container"{
            @for (i,song) in list.iter().enumerate(){
                {CompiledSong{song : song.into(),id: Some(i as u64)}}
            }
        }
    }
    
    Song<'i>(song: &'i base::Song){
        article . "u-song" [
                "data-tonic" = song.tonality.tonic,
                "data-otonic" = song.tonality.tonic,
                "data-stonic" = song.tonality.tonic,
                "data-id" = &song.id,
                //"data-orders" = get_orders_serialized(song), /* back to the drawing board with this feature */
                "data-mode" = {
                    if let base::TonicKind::Minor = song.tonality.kind{
                        "m"
                    }else{
                        ""
                    }
                },
                "data-bpm" = song.bpm,
                "data-nsections" = song.sections.len()
            ]{
            {SongHeader{name : &song.name, tonality : song.tonality}}
            @if let Some((_name,indexes)) = get_default_order(song){
                {SongOrder{
                    sections : &song.sections,
                    order : indexes,
                    use_flats: base::get_default_use_flat(song.tonality)
                }}
            }else{
                @for sblock in &song.sections{
                    {SongBlock{sblock, use_flats : base::get_default_use_flat(song.tonality)}}
                }
            }
        }

    }
    
    CompiledSong<'i>(song: base::SongRef<'i>, id : Option<u64>){
        article . "u-song"[
                "data-tonic" = song.tonality.tonic,
                "data-otonic" = song.tonality.tonic,
                "data-stonic" = song.source_tonality.tonic,
                "data-id" = &song.id,
                "data-bpm" = song.bpm,
                "data-mode" = {
                    if let base::TonicKind::Minor = song.tonality.kind{
                        "m"
                    }else{
                        ""
                    }
                },
                "data-song-id" = {
                    if let Some(id) = id{
                        format!("{:X}",id)
                    }else{
                        "".to_string()
                    }
                },
                "data-nsections" = song.sections.len(),
            ]{
            @if !song.headless{
                {SongHeader{name : song.name, tonality : song.tonality}}
            }
            @for sblock in  song.sections{
                {SongBlock{sblock,use_flats : base::get_default_use_flat(song.tonality)}}
            }
        }
    }
    SongHeader<'i>(name : &'i str, tonality : base::Tonality){
        $ "u-title-box"{
            @if !name.is_empty(){
                $ "u-song-name"{
                    h2 {{name}}
                }
            }
            {ToneButton{tonality: *tonality}}
            {UtilButtonsHTML{abc : false}}
        }
    }

    SongOrder<'i,'b>(sections : &'i Vec<base::SongBlock>,order : &'b Vec<usize>,use_flats : bool){
        @for may_section in order.iter().map(|x|sections.get(*x)){
            @if let Some(base::SongBlock::Section(section)) = may_section{
                {Section{section, use_flats : *use_flats} }
            }else if let Some(base::SongBlock::Annotation(s)) = may_section{
                //TODO:
                {eprintln!("Unimplemented compiled annotation!");""}
            }else{
                {eprintln!("Corrupted section information!");""} //Todo
            }
        }
    }
    
    SongBlock<'i>(sblock : &'i base::SongBlock, use_flats : bool){
        @match sblock{
            base::SongBlock::Section(section) => {
                {Section{section : section,use_flats : *use_flats}}
            }
            base::SongBlock::Annotation(annotation) => {
                $ "u-annotation" {
                    {annotation}
                }
            }
            
        }
    }
    
    Section<'i>(section : &'i base::Section, use_flats : bool){
        @if let Some(tonality) = section.local_tonality{
            $ "u-section" [
                "data-id" = &section.name,
                "data-tonic" = tonality.tonic,
                "data-mode" = {
                        if let base::TonicKind::Minor = tonality.kind{
                            "m"
                        }else{
                            ""
                        }
                    },
                "data-has-abc" = section.has_abc
                ]
            {
                $ "u-title-box"{
                    $ "u-title-background"{
                        $ "u-section-name"{
                            {&section.name}
                        }
                        h3{
                            {&section.description}  
                        }
                        @if section.name.starts_with('C') {
                            $"u-chorus-mark"{}
                        }
                    }
                    {ToneButton{tonality}}
                    {UtilButtonsHTML{abc : section.has_abc}}
                }
                @for subsection in &section.subsections{
                    {Subsection{subsecion : subsection, use_flats : *use_flats}}
                }
            }
        }else{
            $ "u-section" ["data-id" = &section.name, "data-has-abc" = section.has_abc]
            {
                $ "u-title-box"{
                    $ "u-title-background"{
                        $ "u-section-name"{
                            {&section.name}
                        }
                        h3{
                            {&section.description}  
                        }
                        @if section.name.starts_with('C') {
                            $"u-chorus-mark"{}
                        }
                    }
                    {UtilButtonsHTML{abc : section.has_abc}}
                }
                @for subsection in &section.subsections{
                    {Subsection{subsecion : subsection, use_flats : *use_flats}}
                }
            }
        }
    }
    Subsection<'i>(subsecion : &'i base::Subsection, use_flats : bool){
        $ "u-s"[
            "data-abc"= &subsecion.abc
        ]{
            @if let Some(re) = subsecion.metadata.get("ref") {
                $"u-ref"{
                    {re}
                }
            }
            @for line in &subsecion.lines{
                {Line{line, use_flats : *use_flats}}
            }
        }
    }
    Line<'i>(line : &'i base::Line, use_flats : bool){
        @match line {
            base::Line::Lyrics(lyrics)=>{
                $"u-xl" {
                @for measure in lyrics{
                    $"u-m"{
                        @for block in measure{
                            $"u-b"{
                            $"u-l"{
                           
                                @for fragment in block{
                                    {LyricEvent{evt : fragment}}
                                }
                            }
                            }
                        }
                    }
                }
                }
            }
            base::Line::Chords(chords)=>{
                @let mut on_parens = false;
                $"u-xc" {
                @for measure in chords{
                    $"u-m"{
                        @for block in measure{
                            $"u-b"{
                                $"u-c"{
                                @for fragment in block{
                                    @if let base::MusicEvent::OpenParen = fragment{
                                        {on_parens = true;""}
                                    }else if let base::MusicEvent::CloseParen = fragment{
                                        {on_parens = false;""}
                                    }
                                    {MusicEvent{evt : fragment, use_flats : *use_flats, is_par: on_parens}}
                                }
                                }
                            }
                        }
                    }
                }
                }
            }
            base::Line::Mixed(elems)=>{
                @let mut on_parens = false;
                $"u-x" {
                @for measure in elems{
                    $"u-m"{
                        @for (chord_block,lyric_block) in measure{
                            $"u-b"{
                                @if chord_block.is_empty(){
                                    $"u-ce"{}
                                }else{
                                    $"u-c"{
                                        @for evt in chord_block{
                                            @if let base::MusicEvent::OpenParen = evt{
                                                {on_parens = true;""}
                                            }else if let base::MusicEvent::CloseParen = evt{
                                                {on_parens = false;""}
                                            }
                                            {MusicEvent{evt, use_flats : *use_flats,is_par : on_parens}}
                                        }
                                    }
                                }
                                $"u-l"{
                                    @for evt in lyric_block{
                                        {LyricEvent{evt}}
                                    }
                                }
                            }
                        }
                        }
                }
                }
            }
        }
    }

    TimeOffset<'i>(time : &'i base::TimeOffset){
        $ "u-tim"{
            @if time.beat == -1{
                @if time.den == 2 && time.num == 1{
                    {"<"}
                }else{
                    span {
                        "-"
                        sup {{time.num}}
                        sub {{time.den}}
                    }
                }
            }else{
                { time.beat }

                @if time.den == 0 || time.num == 0{

                }else if time.den == 2 && time.num == 1{
                    {"\""}
                }else{
                    span {
                        sup {{time.num}}
                        sub {{time.den}}
                    }
                }
            }
        }
    }
    
    LyricEvent<'i>(evt: &'i base::LyricEvent){
        @match evt{
            base::LyricEvent::LyricText(text)=>{
                {text}
            }
            base::LyricEvent::LyricBreak=>{
                $"u-lbrk"{
                    "\u{200B}"
                }
            }
        }
    }

    MusicEvent<'i>(evt : &'i base::MusicEvent, use_flats : bool, is_par : bool){
        @match evt{
            base::MusicEvent::ChordEvent(evt)=>{
                @if !*is_par{
                    $ "u-h"{
                        {ChordEvent{chord : evt, use_flats : *use_flats}}
                    }
                }else{
                    $ "u-h" . "p"{
                        {ChordEvent{chord : evt, use_flats : *use_flats}}
                    }
                }
            }
            base::MusicEvent::MelodyEvent(evt)=>{
                $ "u-mel"{
                    {MelodyEvent{melody:evt, use_flats : *use_flats}}
                }
            }
            base::MusicEvent::RepeatMeasure=>{
                $ "u-sym" . "rept"{
                    {"/"} //TODO: ooo
                }
            }
            base::MusicEvent::Annotation(text)=>{
                $ "u-ann"{
                    {text}
                }
            }
            base::MusicEvent::NumberedMeasure(num)=>{
                $ "u-num"{
                    {num}
                }
            }
            base::MusicEvent::OpenParen=>{
                $ "u-sym" . "opp"{
                    "("
                }
            }
            base::MusicEvent::CloseParen=>{
                $ "u-sym" . "clp"{
                    ")"
                }
            }
            _=>{
                {eprintln!("Not implemented {:?}!",evt);""} //TODO ooo
            }
        }
    }

    ChordEvent<'i>(chord : &'i base::ChordEvent, use_flats : bool){
    
        @if let Some(time) = &chord.time {

                {TimeOffset{time}}
        }

        $ "u-r" {
            {ChordRoot{root : chord.root,use_flats : *use_flats}}
        }

        @if let base::ChordKind::Minor = chord.kind{
            $ "u-n"{"m"}
        }
        @for modifier in &chord.modifiers{
            {ChordModifier{modifier}}
        }
        @if let Some(bass) = chord.bass{
            $"u-bas" {
                    "/"
                    $ "u-r" {
                    {ChordRoot{root : bass, use_flats: *use_flats}}
                } 
            }
        }
    }

    MelodyEvent<'i>(melody : &'i Vec<base::NoteHeight>, use_flats : bool){

        @for (i,note) in melody.iter().enumerate(){
            $"u-r"{
                {ChordRoot{root:*note,use_flats : *use_flats}} 
            }
            {if i < melody.len() - 1 {" "} else {""} }
        }
    }

    ChordModifier<'i>(modifier : &'i base::ChordModifier){
        @match modifier{
            base::ChordModifier::Keyword(keyword) =>{
                @match keyword{
                    base::ChordKeyword::Maj |
                     base::ChordKeyword::K5 |
                     base::ChordKeyword::K6 |
                     base::ChordKeyword::K7 |
                     base::ChordKeyword::K9 | 
                     base::ChordKeyword::K13 => {
                        //Short for keyword
                        $"u-k" {{strings::CHORD_KEYWORDS_NAMES[keyword]}}
                    }
                    _=>{
                        //Short for keyword-long
                        $"u-kl" {{strings::CHORD_KEYWORDS_NAMES[keyword]}}
                    }
                }
            }
            base::ChordModifier::Alteration(alter)=>{
                $ "u-alt"  {
                    @match alter.kind {
                        base::ChordAlterationKind::Flat =>{"b"}
                        base::ChordAlterationKind::Sharp => {"#"}
                        base::ChordAlterationKind::No => {"no"}
                    }
                    {alter.degree}                   
                }
                
            }
        }
    }
    
    Tonality(tonality : base::Tonality){
        {ChordRoot{root : tonality.tonic, use_flats : base::get_default_use_flat(*tonality)}}
        @if let base::TonicKind::Minor = tonality.kind{
            "m"
        }
    }
    
    ChordRoot(root : base::NoteHeight, use_flats : bool){
        @if base::is_altered(*root){
            @if *use_flats{
                {base::FLAT_TONIC_NAMES[*root as usize]}
                $"u-a" {
                    "b"
                }
            }else{
                {base::SHARP_TONIC_NAMES[*root as usize]}
                
                $"u-a"{
                    "#"
                }
            }
        }else{
            {base::SHARP_TONIC_NAMES[*root as usize]}
        }
    }
    ToneButton(tonality : base::Tonality){
        button . "tone-button" {
                {ChordRoot{root : tonality.tonic, use_flats : base::get_default_use_flat(*tonality)}}
                @if let base::TonicKind::Minor = tonality.kind{
                    $"u-n"{"m"}
                }
        }
    }
    
    UtilButtonsHTML(abc : bool){
        button . "open-util-box"{
            "▶"
        }
        span . "util-buttons-box"{
            button . "collapse-button" {

            }
            button ."moveup-button"{

            }
            button . "movedown-button"{

            }
            button . "remove-button" {

            }
        }
    }
);
fn get_default_order(song : &base::Song)->Option<(&str, &Vec<usize>)>{
    song.orders.get_key_value("default")
    .or_else(
        || song.orders.get_key_value("full")
    ).or_else(
        || song.orders.iter().next()
    ).map(|(k,v)|(k.as_str(),v))
}