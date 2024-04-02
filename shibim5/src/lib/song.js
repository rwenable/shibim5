import {clear_abc, update_abc} from "./abc"
var sharp_setting = null;
let transpose_data = {
	sharps : null,
	transpose : 0,
	song_delta : null,
	section_delta : null
}
export const NT_s = ["C",
    "C<u-a>#</u-a>",
    "D", "D<u-a>#</u-a>",
    "E", "F",
    "F<u-a>#</u-a>", "G",
    "G<u-a>#</u-a>", "A",
    "A<u-a>#</u-a>",
    "B", "C"];
export const NT_b = ["C", "D<u-a>b</u-a>",
    "D", "E<u-a>b</u-a>",
    "E", "F",
    "G<u-a>b</u-a>",
    "G", "A<u-a>b</u-a>",
    "A", "B<u-a>b</u-a>",
    "B", "C"];
export const NT_kv = {
    "c": 0,
    "d": 2,
    "e": 4,
    "f": 5,
    "g": 7,
    "a": 9,
    "b": 11
}

let presentation_channel = false;
let websocket;

function move_tonic_auto(note, tgt, src, sharp = null) {
    if (sharp == null) {
        sharp = true;
        if (tgt == 3 | tgt == 5 | tgt == 8 | tgt == 10) {
            sharp = false;
        }
    }
    var delta = (tgt - src + 12) % 12;
    if (sharp) {
        return NT_s[(delta + note) % 12];
    } else {
        return NT_b[(delta + note) % 12];
    }
}

function find_tonic(el) {
    if (el.hasAttribute("data-tonic")) {
        let root = parseInt(el.getAttribute("data-tonic"), 10);
        if (el.getAttribute("data-mode") == "m") {
            root = (root + 3) % 12
        }
        return root;
    } else if (el.parentElement) {
        return find_tonic(el.parentElement);
    } else {
        return 0;
    }
}
function offset_notes(el, tgt_tonic, src_tonic = null, sharps = null){
    if (src_tonic == null) {
        src_tonic = find_tonic(el);
    }
    change_note(el, (src_tonic + tgt_tonic) %12, src_tonic, sharps);
}
function change_sharps(el, sharps = null){
    if (sharps == null) {
        let src_tonic = find_tonic(el);
        change_note(el, src_tonic, src_tonic, null);
    }else{
        change_note(el, 0, 0, sharps);
    }
}
function change_note(el, tgt, src = null, sharps = null) {
    if (src == null) {
        src = find_tonic(el);
    }
    var elems = Array.from(el.querySelectorAll("u-r"));
    //console.log(elems);
    for (var i = 0; i < elems.length; i++) {
        let base = parseToneButtonString(elems[i].textContent);
        elems[i].innerHTML = move_tonic_auto(base, tgt, src, sharps);
    }
    var buttons = Array.from(el.getElementsByClassName("tone-button"));
    for (var button of buttons) {
        var old_str = button.textContent.trim();
        var old = parseToneButtonString(old_str);
        var end = "";
        if (old_str.slice(-1) == "m") {
            console.warn("FIXME base = (src + 9)");
            //let base = (src + 9) % 12;
            end = "m";
        }
        button.innerHTML = move_tonic_auto(old, tgt, src, sharps) + end;
    }
    el.setAttribute("data-tonic", el.getAttribute("data-mode") == "m" ? (tgt + 9) % 12 : tgt);
    var descendents = Array.from(el.querySelectorAll("[data-tonic]"));
    for (var sec of descendents){
        let cur = parseInt(sec.getAttribute("data-tonic"),10);
        let newc = (tgt - src) + cur;
        sec.setAttribute("data-tonic", sec.getAttribute("data-mode") == "m" ? (newc + 9) % 12 : newc);
    }
    if (tgt - src !== 0){
        clear_abc(el);
        update_abc(el);
    }
}
function parseToneButtonString(str){
    let base = NT_kv[str.slice(0,1).toLowerCase()]
    if (str[1] == "#") {
        base = (base + 1) % 12
    }
    if (str[1] == "b") {
        base = (base + 11) % 12;
    }
    return base
}
//Saving
export function saveString(string, filename) {
    var file, link;

    file = new Blob([string], { type: 'text/html' });
    link = document.createElement("A");
    link.download = filename;
    link.href = window.URL.createObjectURL(file);
    link.style.display = "none";

    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
}


function getCSSString(stylesheet) {
    var str = "";
    for (var i = 0; i < stylesheet.rules.length; i++) {
        str += stylesheet.rules[i].cssText + "\n";
    }
    return str;
}

export function docToHTML(target = null, inner = true) {
    const save_html_head =
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"></head><body>";
    const save_html_close = "</body></html>";
    if (target == null) {
        target = document.body.cloneNode(true);
        Array.from(target.querySelectorAll("[data-forget-save]")).forEach((el) => {
            el.remove();
        });
    }
    let text = inner ? target.innerHTML : target.outerHTML;
    return save_html_head + Array.from(document.styleSheets).map(v => "<style>" + getCSSString(v) + "</style>").join("") +
        text + save_html_close;
}

async function savePage() {
    await fetch_presentation_html(document.getElementById("pres-btn").getAttribute("href"));
    var strname = location.href.split("/").slice(-1)[0];
    if (strname.slice(-5) != ".html") {
        strname += ".html";
    }
    saveString(docToHTML(null, true), strname);
}

function toggleEditable(target = null) {
    if (!target) {
        target = document.body;
    }
    let edit = target.getElementsByClassName("u-song");
    for (var i = 0; i < edit.length; i++) {
        toggleAttr(edit[i], "data-hidecontrol");
        toggleAttr(edit[i], "contenteditable");
    }
}
//Set Events
function setGlobalButtons(target = null) {
    if (!target) {
        target = document.body;
    }
    document.getElementById("save-btn")
        .addEventListener("click", () => {
            document.getElementById("nav-bar").style.display = "none";
            savePage();
        });

    document.getElementById("copy-btn")
        .addEventListener("click", () => {
            let text = get_lyrics(document.body);
            navigator.clipboard.writeText(text);
        });
    document.getElementById("big-btn")
        .addEventListener("click", () => changeFontSize(target, 1, 1.1));
    document.getElementById("small-btn")
        .addEventListener("click", () => changeFontSize(target, -1, 0.91));
    document.getElementById("hidechord-btn")
        .addEventListener("click", function () {
            toggleAttr(target, "data-hidechord");
            this.toggleAttribute("data-enabled");
        });
    document.getElementById("hidelyric-btn")
        .addEventListener("click", function () {
            toggleAttr(target, "data-hidelyric");
            this.toggleAttribute("data-enabled");
        });
    document.getElementById("hidecontrol-btn")
        .addEventListener("click", function () {
            toggleEditable();
            this.toggleAttribute("data-enabled");
        });
    document.getElementById("margin-btn")
        .addEventListener("click", function () {
            Array.from(document.getElementsByClassName("u-container")).forEach(a => {
                a.toggleAttribute("data-nomargin");
            });
            this.toggleAttribute("data-enabled");
        });
    document.getElementById("pres-btn")
        .addEventListener("click", function (evt) {
            if (location.protocol === "file:") {
                let str = get_data_div("presentation_html");
                let new_w = window.open(undefined,"_blank","popup");
                new_w.document.write(str);
                evt.preventDefault();
            }
        });
    document.getElementById("sharp-btn").addEventListener("click", ()=>{
        sharp_setting = !sharp_setting;
        transpose_all(0);
    });
    document.getElementById("tr-up-btn").addEventListener("click",()=>{
        transpose_all(1);
    });
    document.getElementById("tr-down-btn").addEventListener("click",()=>{
        transpose_all(-1);
    });
    setToolbarHide();
}
export function setSharpSetting(val,head){
    sharp_setting = val;
    transpose_all(0,head);
}
function encodeSearchParams(obj){
    let params = [];
    if(obj.transpose){
        params.push(["t",obj.transpose.toString()]);
    }
    if(obj.song_delta){
        for(let song of obj.song_delta){
            params.push(["t"+song[0].toString(),song[1].toString()])
        }
    }
    return params;
}
function extractTransposeParams(root){
    let counter = document.getElementById("tr-current");
    let obj = {}
    let delta = parseInt(counter.textContent,10);
    if (delta || delta === 0) {
        obj.transpose = delta;
    }else{
        console.error("Invalid text in transpose value");
    }
    obj.song_delta = [];
    let songs = Array.from(root.getElementsByClassName("u-song"));
    songs.forEach((elem)=>{
        let tonic = parseInt(elem.getAttribute("data-tonic"));
        let otonic = parseInt(elem.getAttribute("data-otonic"));
        let song_id = elem.getAttribute("data-song-id");    
        let otonic_tr = (otonic + delta + 12) % 12;
        let song_delta = (tonic - otonic_tr+ 12) %12;
        if (song_delta !== 0){
            obj.song_delta.push([song_id,song_delta]);
        }
    });
    return obj;
}

function updateTransposeHistory(){
    console.log("Not implemented")
    //TODO
    return
    let params = encodeSearchParams(extractTransposeParams(document.body)).map(([n,v])=> n + "=" + v).join("&");
    let url = new URL(window.location.pathname,window.location.origin);
    url.search = params;
	history.pushState({},"",url);
}
function setToneButton(btn) {
    btn.addEventListener("click", function () {
        var song = this.parentElement.parentElement;
        var bar = song.getElementsByClassName("tone-bar")[0];
        if (bar) {
            song.removeChild(bar);
        } else {
            var section1 = song.getElementsByTagName("u-title-box")[0];
            song.insertBefore(createToneButtons(), section1.nextSibling);
        }
    });
}

/*
function setHideOnScroll(target){
    document.addEventListener("scroll",(evt)=>{
        if (target._prev_scroll){
            if (window.scrollY > target._prev_scroll){
                target.style.display = "none";
            }else{
                target.style.display = "";
            }
        }
        target._prev_scroll = window.scrollY;
    });
}
*/
function setToolbarHide() {
    Array.from(document.getElementsByClassName("u-container")).forEach((cont) => {
        cont.addEventListener("click", () => {
            document.getElementById("nav-bar").style.display = "none";
        });
    });
    document.getElementById("settings-button").addEventListener("click", () => {
        document.getElementById("nav-bar").style.display = "";
    });
    document.getElementById("nav-bar").addEventListener("click", function (evt) {
        if (this === evt.target) {
            document.getElementById("nav-bar").style.display = "none";
        }
    });
    document.body.addEventListener("click", function (evt) {
        if (this === evt.target) {
            document.getElementById("nav-bar").style.display = "none";
        }
    });
}
let key_37_locked = false;
let key_39_locked = false;
function setScrollKeys() {
    document.addEventListener("keydown", event => {
        if (event.keyCode === 37 && !key_37_locked) {
            key_37_locked = true;
            event.preventDefault();
            scrollPrevSongView();
        } else if (event.keyCode === 39 && !key_39_locked) {
            event.preventDefault();
            key_39_locked = true;
            scrollNextSongView();
        }
        // do something
    });

    document.addEventListener("keyup", event => {
        if (event.keyCode === 37) {
            key_37_locked = false;
        } else if (event.keyCode === 39) {
            key_39_locked = false;
        }
    });
}

function setLocalButtons(head) {
    var exp = head.getElementsByClassName("collapse-button")
    for (var i = 0; i < exp.length; i++) {
        exp[i].addEventListener("click", function () {
            var song = this.closest("u-section, article");
            toggleAttr(song, "data-collapsed");
        });
    }

    var up = head.getElementsByClassName("moveup-button")
    for (var i = 0; i < up.length; i++) {
        up[i].addEventListener("click", function () {
            var root = this.closest("u-section, article");
            swapPrev(root);
            root.classList.add("notice");
            setTimeout(() => root.classList.remove("notice"), 1000);
            if (root.tagName == "ARTICLE") {
                wait_DOM(function () {
                    if (window.innerHeight > 800) {
                        root.scrollIntoView({ behavior: "instant", block: "nearest" });
                    } else {
                        root.scrollIntoView({ behavior: "instant", block: "start" });
                    }
                });
            }
        });
    }

    var down = head.getElementsByClassName("movedown-button")
    for (var i = 0; i < down.length; i++) {
        down[i].addEventListener("click", function () {
            var root = this.closest("u-section, article");
            swapNext(root);
            root.classList.add("notice");
            setTimeout(() => root.classList.remove("notice"), 1000);
            if (root.tagName == "ARTICLE") {
                wait_DOM(function () {
                    root.scrollIntoView({ behavior: "instant", block: "nearest" });
                });
            }
        });
    }

    var edit = head.getElementsByClassName("remove-button")
    for (var i = 0; i < edit.length; i++) {
        edit[i].addEventListener("click", function () {
            let del = this.closest("u-section, article");
            del.parentElement.removeChild(del);
        });
    }

    var songs = head.getElementsByClassName("u-song");

    Array.from(head.getElementsByClassName("tone-button")).forEach(setToneButton);

}

function createToneButtons() {
    let div = document.createElement("div");
    div.className = "tone-bar";
    function mkbtn(i) {
        let btn = document.createElement("button");
        btn.innerHTML = move_tonic_auto(0, i, 0, sharp_setting) + "<br/>" +
            String(move_tonic_auto(9, i, 0, sharp_setting) + "m");
        btn.addEventListener("click", function () {
            var song = this.parentNode.parentNode.parentNode;
            change_note(song, i, null, sharp_setting);
            updateTransposeHistory();
            song.removeChild(this.parentNode.parentNode);
        });
        return btn;
    }
    let span1 = document.createElement("span");
    let span2 = document.createElement("span");
    for (var i = 0; i < 6; i++) {
        span1.appendChild(mkbtn(i));
    }
    for (var i = 6; i < 12; i++) {
        span2.appendChild(mkbtn(i));
    }
    div.appendChild(span1);
    div.appendChild(span2);
    return div;
}
async function fetch_presentation_html(href) {
    let local = get_data_div("presentation_html");
    if (!local) {
        let res = await fetch(href);
        let str = await res.text();
        store_data_div("presentation_html", str);
        return str;
    }
    return local;
}

function store_data_div(name, data) {
    let pdiv = document.getElementById(name);
    if (!pdiv) {
        let div = document.createElement("DIV");
        div.id = name;
        div.style.display = "none";
        div.setAttribute("data-content", data);
        document.body.appendChild(div);
    } else {
        pdiv.setAttribute("data-content", data);
    }
}

function get_data_div(name) {
    let pdiv = document.getElementById(name);
    if (pdiv) {
        return pdiv.getAttribute("data-content");
    }
    return null;
}

function transpose_all(offset, head = null){
    if(!head){
        head = document;
    }
    let songs = Array.from(head.getElementsByClassName("u-song"));
    for (const song of songs){
        offset_notes(song, offset,null, sharp_setting);
    }
    let counter = document.getElementById("tr-current");
    if (counter){
        let val = parseInt(counter.textContent.trim());
        val = (val + offset + 12)%12;
        counter.textContent = val.toString();
        transpose_data.transpose = val;
        transpose_data.sharps = sharp_setting;
    }
    updateTransposeHistory();
}

function check_search_params(){
    const url = new URL(window.location);
    const params = url.searchParams;
    let transpose = parseInt(params.get("t"));
    if (!transpose){
        transpose = 0;
    }else{
        let counter = document.getElementById("tr-current");
        counter.textContent = transpose.toString();
    }
    let songs = Array.from(document.getElementsByClassName("u-song"));
    for (const song of songs){
        let offset = transpose;
        let song_offset = parseInt(params.get("t"+song.getAttribute("data-song-id")));
        if (song_offset){
            offset = (offset + song_offset + 12) %12
        }
        offset_notes(song, offset,null, sharp_setting);
    }
}
function set_up_all() {
    setGlobalButtons();
    setLocalButtons(document.body);
    set_present_channel(document.body);
    setScrollKeys();
    try_default_websocket();
    check_search_params();
}
export function set_up_local(el) {
    //setGlobalButtons();
    setLocalButtons(el);
    //set_present_channel(el);
    //setScrollKeys();
    //try_websockets();
    //check_search_params();
}
/*
if (document.readyState === "interactive" || document.readyState === "complete") {
    set_up_all();
} else {
    document.addEventListener("DOMContentLoaded", set_up_all);
}*/

//Utils
function toggleAttr(song, attr) {
    if (song.hasAttribute(attr)) {
        song.removeAttribute(attr);
    } else {
        song.setAttribute(attr, "");
    }
}

function changeFontSize(target, delta = 1, alpha = 1) {
    let fontsiz = parseFloat(window.getComputedStyle(target, null).getPropertyValue('font-size'));
    let new_size = Math.round(fontsiz * alpha + delta);
    if (new_size < 10) {
        new_size = 10;
    }
    if (!target) {
        target = document.body;
    }
    target.style.fontSize = (new_size) + 'px';
}

function swapPrev(elem) {
    var sib = elem.previousSibling
    if (sib && sib.tagName == elem.tagName) {
        elem.parentElement.insertBefore(elem, sib);
    }
}
function swapNext(elem) {
    var sib = elem.nextSibling
    if (sib && sib.tagName == elem.tagName) {
        elem.parentElement.insertBefore(sib, elem);
    }
}
function wait_DOM(fn) {
    let step = function () { window.requestAnimationFrame(fn) }
    window.requestAnimationFrame(step);
}

export function set_present_channel(target) {
    if (!target){
        target = document.body;
    }
    if (!presentation_channel) {
        presentation_channel = new BroadcastChannel("presentation");
        let presentation_head = target;
        target.setAttribute("data-present", "");
        presentation_channel.onmessage = (msg) => {
            let data = msg.data;
            if (data.type == "REQUEST_ACTION") {
                switch (data.payload) {
                    case "NEXT_SECTION":
                        selectSectionToPresent(findNextSelection(1, presentation_head));
                        break;
                    case "PREV_SECTION":
                        selectSectionToPresent(findNextSelection(-1, presentation_head));
                        break;
                }
            }
        };
    }
}


function findNextSelection(offset, head) {
    let collection = Array.from(head.getElementsByTagName("u-s"));
    collection = collection.filter(elem => !elem.parentElement.hasAttribute("data-collapsed"));
    if (collection.length == 0) {
        return;
    }
    offset = offset % collection.length;
    let last_idx = collection.findIndex(e => e.classList.contains("u-selected"))
    if (last_idx < 0) {
        return collection[0];
    }
    let new_index = (last_idx + offset + collection.length) % collection.length;
    return collection[new_index];
}

export function setPresentationClick(head) {
    if (presentation_channel) {
        let sections = head.getElementsByTagName("u-s");
        for (var i = 0; i < sections.length; i++) {
            let elem = sections[i];
            elem.addEventListener("click", () => {
                selectSectionToPresent(elem);
            });
        }
    }
}
export function setPresFontSize(size){
    if (websocket && websocket.readyState == WebSocket.OPEN) {
        //websocket.send("HTM#" + message);
    } else if (presentation_channel) {
        presentation_channel.postMessage({
            type: "FONTSIZE",
            payload: size
        });
    }
}

function selectSectionToPresent(elem) {
    let last = document.querySelector("u-s.u-selected");
    if (last) {
        last.classList.remove('u-selected');
    }
    elem.classList.add('u-selected');
    let message = filterSectionToPresent(elem).innerHTML;
    if (websocket && websocket.readyState == WebSocket.OPEN) {
        websocket.send("HTM#" + message);
    } else if (presentation_channel) {
        presentation_channel.postMessage({
            type: "CONTENT",
            payload: message
        });
    }
}

function filterSectionToPresent(elem) {

    let out = document.createElement("section");
    out.className = "u-section";

    let lines = elem.querySelectorAll("u-x, u-xl");
    for (let i = 0; i < lines.length; i++) {
        let old_line = lines[i].cloneNode(true);
        old_line.querySelectorAll("u-c").forEach((el) => {
            el.parentElement.removeChild(el)
        });

        let new_line = document.createElement("div");
        new_line.className = "u-l";
        let new_text = old_line.textContent.replace(/-/g, "");
        let blocks = sepByDelims(new_text).map(str => {
            let sp = document.createElement("SPAN");
            sp.textContent = str;
            return sp;
        }).forEach(block => {
            new_line.appendChild(block);
        });
        out.appendChild(new_line);
    }
    let ref = elem.getElementsByTagName("u-ref");
    if (ref.length > 0) {
        let sp = document.createElement("DIV");
        sp.className = "u-ref";
        sp.textContent = ref[0].textContent;
        out.appendChild(sp);
    }
    return out;
}

function get_lyrics(elem) {
    let out = "";
    let paragraphs = elem.getElementsByTagName("u-s");
    for (let i = 0; i < paragraphs.length; i++) {
        let lines = paragraphs[i].getElementsByTagName("u-x");
        for (j = 0; j < lines.length; j++) {
            let texts = lines[j].getElementsByTagName("u-l");
            for (k = 0; k < texts.length; k++) {
                out += texts[k].textContent;
            }
            out += "\n";
        }
        out += "\n";
    }
    return out;
}

function sepByDelims(text) {
    let try0 = text.split("\u200B");
    if (try0.length > 1) {
        return try0;
    }
    let try1 = text.match(/[^;,.:?!]+[;,.:?!]* ?/g);
    if (try1 && (try1.length > 1 || text.length < 32)) {
        return try1;
    }
    let try2 = text.match(/[^ ]+[ ]?/g);
    if (!try2 || try2.length < 2) {
        if (try1) {
            return try1;
        } else {
            return try0;
        }
    }
    let mid = Math.floor(try2.length / 2);
    return [try2.slice(0, mid).join(""), try2.slice(mid, try2.length).join("")];
}

export function scrollNextSongView() {
    let titles = Array.from(document.getElementsByClassName("u-song"));
    if (titles.length < 1) {
        return;
    } else if (titles.length < 2) {
        getNextSection(titles[0]).scrollIntoView({ block: "start", inline: "start" });
        return;
    }
    let idx = titles.findIndex(el => el.offsetTop - 10 > window.scrollY);
    if (idx === -1) {
        idx = titles.length - 1;
    }
    if (idx === 0){
        titles[0].scrollIntoView({ block: "start", inline: "start" });
        return;
    }
    let prev = titles[idx - 1];
    let remaining_h = prev.offsetTop + prev.scrollHeight - (window.scrollY + window.innerHeight);

    if (remaining_h > 10) {
        getNextSection(prev).scrollIntoView({ block: "start", inline: "start" });
    } else {
        titles[idx].scrollIntoView({ block: "start", inline: "start" });
    }
}
function getNextSection(song) {
    let subsc = Array.from(song.getElementsByTagName("u-section"));
    let r = subsc.findIndex(
        el => el.offsetTop + el.scrollHeight > window.scrollY + window.innerHeight);
    if (r < 0) {
        r = subsc.length - 1;
    }
    return subsc[r];
}
export function scrollPrevSongView() {
    //Todo: step on sections if song(s) is too long?
    let titles = Array.from(document.getElementsByClassName("u-song"));
    if (titles.length < 1) {
        return;
    }
    let idx = titles.findIndex(el => el.offsetTop + 10 > window.scrollY);
    if (idx === -1 && titles.length > 1) {
        idx = titles.length - 2;
    } else if (idx > 0) {
        idx = idx - 1;
    } else {
        idx = 0;
    }
    titles[idx].scrollIntoView({ block: "start", inline: "start" });
}

export async function try_default_websocket(){
    let ws = await create_websocket(null,(sucess,ws)=>{
        if(sucess){
            websocket = ws;
        }
    })
        .catch(async ()=>create_websocket("ws://localhost:64064/ws",(sucess,ws)=>{
            if(sucess){
                websocket = ws;
            }
        }));
    websocket = ws;
    return ws;
}

export function set_websocket(ws){
    if(websocket){
        websocket.close();
        if (websocket.tid){
            clearTimeout(websocket.tid);
        }
    }
    websocket = ws;
}
export function create_websocket(uri = null, retry_callback = null) {
    return new Promise( (resolve,reject)=>{
        if (!uri){
            let protocol = window.location.protocol == "http:" ? "ws:" : "wss:";
            uri = protocol + "//" + window.location.host + "/ws";
        }
        let websocket = new WebSocket(uri);
        websocket.onerror = (e)=> reject(e);
        function timeout (){
            websocket.close();
            reject("Timeout");
        }
        function retry() {
            console.log("Retrying websockets");
            let n_tid;
            n_tid = setTimeout( () => {
                websocket = new WebSocket(uri);
                websocket.tid = n_tid;
                websocket.onerror = () => {
                    if(retry_callback && !retry_callback(false,websocket)){
                    }else{
                        websocket.close();
                        retry();
                    }
                };
                websocket.onopen = () => {
                    if(retry_callback){
                        retry_callback(true,websocket)
                    }
                    clearTimeout(n_tid);
                    console.log("Websockets enabled @ "+uri);
                }
                websocket.onclose = websocket.onerror;
            },5000);
        }
        let c_tid = setTimeout(timeout,4500);
        websocket.onopen = () => {
            clearTimeout(c_tid);
            console.log("Websockets enabled @ "+uri);
            websocket.onerror = retry;
            websocket.onclose = retry;
            resolve(websocket);
        }

    });
}

