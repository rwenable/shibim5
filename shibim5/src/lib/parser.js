export class SHBStreamTokenizer{
    static S_META_NAME = 0;
    static S_META_VALUE = 1;
    static S_SECTION_ID = 2;
    static S_SECTION_DESC = 3;
    static S_LINE = 4;
    static singleton = new SHBStreamTokenizer();
    startState(){
        return {s:SHBStreamTokenizer.S_META_NAME};
    }

    token(stream, state){
        if (stream.eatSpace()){
            return null;
        }
        switch (state.s){
            case SHBStreamTokenizer.S_META_NAME:
                if(stream.match(/^[^@:\s]+/)){
                    return "attributeName";
                }
                if(stream.eat(":")){
                    state.s = SHBStreamTokenizer.S_META_VALUE;
                    return "punctuation";
                }
                if(stream.eat("@")){
                    state.s = SHBStreamTokenizer.S_SECTION_ID;
                    return "punctuation"
                }
                stream.next();
                return null;
            case SHBStreamTokenizer.S_META_VALUE:
                if(stream.match(/^[^@:\n]+/)){
                    state.s = SHBStreamTokenizer.S_META_NAME;  
                    return "attributeValue";
                }
                stream.next();
                return null;
            case SHBStreamTokenizer.S_SECTION_ID:
                if(stream.match(/^[^@:\s]+/)){
                    if(stream.eol()){
                        state.s = SHBStreamTokenizer.S_LINE;
                    }else{
                        state.s = SHBStreamTokenizer.S_SECTION_DESC;
                    }
                    return "name";
                }
                
                stream.next();
                return null;
            case SHBStreamTokenizer.S_SECTION_DESC:
                if(stream.match(/^[^@\n]+/)){
                    state.s = SHBStreamTokenizer.S_LINE;
                    return "heading";
                }
                stream.next();
                return null;
            case SHBStreamTokenizer.S_LINE:
                if(stream.sol() && stream.eat("@")){
                    state.s = SHBStreamTokenizer.S_SECTION_ID;
                    return "punctuation";
                }
                if(stream.sol() && stream.match(/^[ ]*--!.*/)){
                    return "comment";
                }
                if(stream.match(/^[^·^´`|*]+[*·]/,false)){
                    stream.match(/^[^·^´`|*]+/);
                    return "literal"
                }
                if(stream.eat(/[·*]/)){
                    return "unit"
                }
                if(stream.eat(/[|`´]/)){
                    return "separator"
                }
                stream.next();
                return "content";
            default:
                console.log(state);
                stream.next();
                return null;
        }
    }
}

export class LSTStreamTokenizer{
    static S_SONG_NAME = 0;
    static S_META_NAME = 1;
    static S_META_VALUE = 2;
    static S_INLINE_START = 3;
    static S_INLINE_META_NAME = 4;
    static S_INLINE_META_VALUE = 5;
    static S_INLINE_CONTENT = 6;
    static S_INLINE_CONTENT_BEGIN = 7;
    startState(){
        return {s:LSTStreamTokenizer.S_SONG_NAME};
    }
    
    token(stream, state){
        if (stream.eatSpace()){
            if(stream.eol()){
                switch(state.s){
                    case LSTStreamTokenizer.S_META_NAME:
                    case LSTStreamTokenizer.S_META_VALUE:
                        state.s = LSTStreamTokenizer.S_SONG_NAME;
                        break;
                    case LSTStreamTokenizer.S_INLINE_META_NAME:
                    case LSTStreamTokenizer.S_INLINE_META_VALUE:
                        state.s = LSTStreamTokenizer.S_INLINE_CONTENT_BEGIN;
                        break;
                    default:
                        break;
                }
            }
            return null;
        }
        switch (state.s){
            case LSTStreamTokenizer.S_SONG_NAME:
                if(stream.sol() && stream.match(/^<</)){
                    state.s = LSTStreamTokenizer.S_INLINE_META_NAME;
                    return "punctuation"
                }
                if(stream.eat('#')){
                    stream.skipToEnd();
                    return "comment"
                }
                
                if(stream.match(/^(\s*[^|/\s]+)+\s*/)){
                    return "link";
                }
                if(stream.eat("|") || stream.eat("/")){
                    state.s = LSTStreamTokenizer.S_META_NAME;
                    if (stream.eol()){
                        state.s = LSTStreamTokenizer.S_SONG_NAME;
                    }
                    return "punctuation";
                }
                stream.next();
                return null;
            case  LSTStreamTokenizer.S_META_NAME:
                if(stream.match(/^[^:|/\s]+/)){
                    if (stream.eol()){
                        state.s = LSTStreamTokenizer.S_SONG_NAME;
                    }
                    return "keyword";
                }
                if (stream.eat(":")){
                    state.s = LSTStreamTokenizer.S_META_VALUE;
                    return "punctuation";
                }
                stream.next();
                return null;
            case LSTStreamTokenizer.S_META_VALUE:
                if(stream.match(/^(\s*[^|/\s]+)+/)){
                    if(stream.eol()){
                        state.s = LSTStreamTokenizer.S_SONG_NAME;
                    }
                    return "attributeValue";
                }
                if(stream.eat("|")){
                    state.s = LSTStreamTokenizer.S_META_NAME;
                    if (stream.eol()){
                        state.s = LSTStreamTokenizer.S_SONG_NAME;
                    }
                    return "punctuation";
                }
                stream.next();
                return null;
            case LSTStreamTokenizer.S_INLINE_META_NAME:
                if (stream.match(/^[^|/:]/)){
                    if (stream.eol()){
                        state.s = LSTStreamTokenizer.S_INLINE_CONTENT_BEGIN;
                    }
                    return "keyword";
                }
                if (stream.eat(":")){
                    state.s = LSTStreamTokenizer.S_INLINE_META_VALUE;
                    return "punctuation";
                }
                stream.next();
                return null;
            case LSTStreamTokenizer.S_INLINE_META_VALUE:
                if (stream.match(/^[^|/]/)){
                    if (stream.eol()){
                        state.s = LSTStreamTokenizer.S_INLINE_CONTENT_BEGIN;
                    }
                    return "attributeValue";
                }
                if (stream.eat("|") || stream.eat("/")){
                    state.s = LSTStreamTokenizer.S_INLINE_META_NAME;
                    return "punctuation";
                }
                stream.next();
                return null;
            case LSTStreamTokenizer.S_INLINE_CONTENT_BEGIN:
                state.sr = SHBStreamTokenizer.singleton.startState(); //:c
                state.s = LSTStreamTokenizer.S_INLINE_CONTENT;
            case LSTStreamTokenizer.S_INLINE_CONTENT:
                if (stream.sol() && stream.match(/^>>.*/)){
                    state.s = LSTStreamTokenizer.S_SONG_NAME;
                    return "punctuation";                
                }
                let r =  SHBStreamTokenizer.singleton.token(stream,state.sr); //:c
                return r;
            default:
                //console.log(state);
                stream.next();
                return null;
        }
    }
}