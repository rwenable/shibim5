import { default as wasm} from "shibim-js/shibim_js";
import wasm_data from "shibim-js/shibim_js_bg.wasm?url"
export function load_wasm(){
    return wasm(wasm_data);
}