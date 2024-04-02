import { renderAbc, synth, numberOfTunes, type MidiBuffer, TimingCallbacks } from "abcjs";
type SongContext = {
    bpm? : number,
    tonality : {
        root : number,
        minor : boolean
    },
    source_root : number
}
const NOTE_NAMES = ["C","C#","D","Eb","E","F","F#","G","Ab","A","Bb","B"];
function get_context(elem : Element):SongContext{
    let target = elem.closest("[data-tonic]");
    if(!target){
        return {
            tonality : {
                root : 0,
                minor : false
            },
            source_root : 0
        };
    }
    let bpm = target.getAttribute("data-bpm");
    let source_tonic = parseInt(target.getAttribute("data-stonic") || "0");
    return {
        bpm : bpm?parseInt(bpm):undefined,
        tonality : {
            root : parseInt(target.getAttribute("data-tonic") || target.getAttribute("data-otonic") || "0"),
            minor : target.getAttribute("data-mode") == "m"?true:false
        },
        source_root : source_tonic
    }
}
export function update_abc(elem : Element, auto_open : boolean){
    let sections = elem.getElementsByTagName("u-section");
    for (const sect of sections){
        let abc = sect.hasAttribute("data-has-abc");
        if(!abc){
            continue;
        }
        let section_button = false;
        let subs = sect.getElementsByTagName("u-s");
        for(let sub of subs){
            let abc = sub.getAttribute("data-abc");
            if(!abc){
                continue;
            }
            if(!section_button){
                let btn = document.createElement("button");
                btn.className = "abc-button";
                btn.textContent = "♫";
                btn.addEventListener("click",function (){
                    toggle_abc_in_section(sect);
                });
                sect.getElementsByClassName("util-buttons-box")[0].appendChild(btn);
                section_button = true;
            }
            if (abc && (auto_open || sect.hasAttribute("data-show-abc"))){
                let measures = getMeasureElems(sub);
                let div = render_default(abc,get_context(sect),measures);
                sub.appendChild(div);
            }
        }
        if(auto_open){
            sect.setAttribute("data-show-abc","true");
        }
    }
}
export function clear_abc(elem : Element){
    let svgs = Array.from(elem.getElementsByTagName("u-abc"));
    for (let abc of svgs){
        abc.remove();
    }
    let btns = Array.from(elem.getElementsByClassName("abc-button"));
    for (let btn of btns){
        btn.remove();
    }

}
function selectMeasures(measures : Element[]){
    for(let measure of measures){
        measure.classList.add("play")
    }
}
function deselectMeasures(measures : Element[]){
    for(let measure of measures){
        measure.classList.remove("play")
    }
}
function getMeasureElems(sect : Element){
    let out : Element[][] = [];
    let lines = sect.querySelectorAll("u-x, u-xc, u-xl");
    if(lines.length === 0){
        return [];
    }
    let first_line = lines[0].getElementsByTagName("u-m")
    out = (Array.from(first_line)).map(x=>[x]);
    for (let i = 1;i < lines.length; i++){
        let measures = lines[i].getElementsByTagName("u-m");
        if (measures.length === 0){
            continue;
        }
        out[out.length-1].push(measures[0]);
        for(let measure of Array.from(measures).slice(1)){
            out.push([measure]);
        }
    }
    return out;
}
export function render_default(abc_code : string, song_context : SongContext, shb_measures? : Element[][]){
    abc_code = abc_code.replace(/^\s+/,"");
    if (song_context.tonality){
        abc_code = `Q:${song_context.bpm || "1/4=100"}\nK:${NOTE_NAMES[song_context.source_root]}${song_context.tonality.minor?"m":""}\n%%MIDI chordprog 20\n${abc_code}`;
    }
    let delta = (song_context.tonality.root -  song_context.source_root + 12)%12;
    delta = delta > 6?(delta-12):delta;
    let conatiner = document.createElement("u-abc");
    let obj = document.createElement("div");
    let render = renderAbc(obj,abc_code,{ responsive: "resize", staffwidth : 450, /*wrap : {
         minSpacing: 1.8,
         maxSpacing: 2.7,
         preferredMeasuresPerLine: 4 
        },*/
        visualTranspose : delta,
     });
     let prev_elems : HTMLElement[] | null = null;
     let measure_n = 0;
     function select_elems(elems : HTMLElement[]){
        for(let elem of elems){
            elem.setAttribute("fill","#ff0040");
        }
     }
     function deselect_elems(elems : HTMLElement[]){
        for(let elem of elems){
            elem.setAttribute("fill","currentColor");
        }
     }
     function deselectMeasuresLast(elems : Element[][]){
        if(elems.length > measure_n){
            deselectMeasures(elems[measure_n])
        }
     }
     let timing_callback = new TimingCallbacks(render[0],{
        eventCallback : (evt) => {
            if (evt?.measureNumber && shb_measures){
                if(evt.measureNumber > measure_n){
                    measure_n += 1;
                    if (shb_measures.length > measure_n){
                        selectMeasures(shb_measures[measure_n]);
                    }
                    if (shb_measures.length > measure_n - 1){
                        deselectMeasures(shb_measures[measure_n-1]);
                    }
                }
            }
            if (!evt || !evt.elements){
                if(shb_measures){
                    deselectMeasuresLast(shb_measures);
                }
                if (prev_elems){
                    deselect_elems(prev_elems);
                }
                return undefined;
            }
            let current = evt.elements.flat();
            select_elems(current);
            if (prev_elems){
                deselect_elems(prev_elems);
            }
            prev_elems = current;
            return undefined;
        }
     });
    conatiner.appendChild(obj);
    if (synth.supportsAudio()){
        
        let play = document.createElement("button");
        play.className = "abc-play-button";
        play.textContent = "▶️";
        let midi_buffer : MidiBuffer;
        play.addEventListener("click",async ()=>{
            if(shb_measures && shb_measures.length > 0){
                deselectMeasuresLast(shb_measures);
                selectMeasures(shb_measures[0]);
            }
            measure_n = 0;
            if (midi_buffer){
                midi_buffer.stop();
                midi_buffer.start();
                timing_callback.start(0);
                return;
            }
            await synth.activeAudioContext().resume();
            midi_buffer = new synth.CreateSynth();
            await midi_buffer.init({
                visualObj : render[0],
                millisecondsPerMeasure : render[0].millisecondsPerMeasure(),
                options : {
                    midiTranspose : delta,
                    drum : "dddd 80 73 73 73 50 45 45 45",
                }
            });
            await midi_buffer.prime();
            timing_callback.start(0);
            midi_buffer.start();
        });
        let stop = document.createElement("button");
        stop.className = "abc-stop-button";
        stop.textContent = "⏹️";
        stop.addEventListener("click",()=>{
            if(shb_measures){
                deselectMeasuresLast(shb_measures);
            }
            if (midi_buffer){
                midi_buffer.stop()
                timing_callback.stop();
                timing_callback.reset();
            }
        }); 
        conatiner.appendChild(play);
        conatiner.appendChild(stop);
    }
    return conatiner;
}

export function create_or_show_abc_for_section(elem : Element){
    let abc = elem.hasAttribute("data-has-abc")
    if(abc){
        elem.setAttribute("data-show-abc","true");
        if(!elem.getElementsByTagName("u-abc").length){
            let subs = elem.getElementsByTagName("u-s");
            for(let sub of subs){
                let abc = sub.getAttribute("data-abc");
                if (!abc){
                    continue;
                }
                sub.appendChild(render_default(abc,get_context(elem),getMeasureElems(sub)));
            }
        }
    }
}

export function toggle_abc_in_section(elem : Element){
    if(elem.hasAttribute("data-show-abc")){
        elem.removeAttribute("data-show-abc");
    }else{
        create_or_show_abc_for_section(elem);
    }
}