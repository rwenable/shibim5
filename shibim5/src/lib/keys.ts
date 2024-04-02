export let key_codes : Record<number,boolean> = {};
document.addEventListener("keydown",evt=>{
    if (!key_codes[evt.keyCode]){
        let e = new CustomEvent("keydown2",{detail:evt});
        document.dispatchEvent(e);
    }
    key_codes[evt.keyCode] = true;
});
document.addEventListener("keyup",evt=>{
    key_codes[evt.keyCode] = false;
});