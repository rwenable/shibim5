const LIBBSC_HEADER_SIZE = 28;
function create_bsc(){
    let bsc_el = document.getElementById("bsc-wasm");
    let bsc_u8 = window.fflate.decode_base85(bsc_el.textContent.trim());
    let bsc_ab = window.fflate.unzlibSync(bsc_u8).buffer;
    window.libbsc.module({wasmBinary : bsc_ab}).then(runtime=>{
        if(runtime._bsc_init(1) !== 0){
            console.error("Could not init libbsc");
        }else{
            window.libbsc.runtime = runtime;
        }
    });
}
function bsc_compress_u8(u8){
    if(!window.libbsc.runtime){
        create_bsc();
    }
    let bsc = window.libbsc.runtime;
    if(!(u8 instanceof Uint8Array)){
        throw new Error("bsc_serialize only supports Uint8Arrays");
    }
    let ptr = bsc._malloc(u8.length + LIBBSC_HEADER_SIZE);
    bsc.HEAPU8.set(u8,ptr);
    let result = bsc._bsc_compress(ptr,ptr,u8.length,15,128,1,1,1);
    if(result >= 0){
        let out = new Uint8Array(result);
        out.set(new Uint8Array(bsc.HEAPU8.buffer,ptr,result))
        bsc._free(ptr);
        return out;
    }
    throw new Error("bsc_compress failed with code "+result);
}
function bsc_decompress_u8(u8){
    if(!window.libbsc.runtime){
        create_bsc();
    }
    let bsc = window.libbsc.runtime;
    //TODO: check if ints are 32 bits
    let block_info_ptr = bsc._malloc(2*4);
    let ptr = bsc._malloc(u8.length);
    bsc.HEAPU8.set(u8,ptr);
    let info_res = bsc._bsc_block_info(ptr,LIBBSC_HEADER_SIZE,block_info_ptr,block_info_ptr+4,1);
    if(info_res !== 0){
        throw new Error("bsc_block_info failed with code "+info_res);
    }
    let block_info = new Uint32Array(bsc.HEAPU8.buffer,block_info_ptr,2);
    console.log(block_info);
    let ptr_out = bsc._malloc(block_info[1]);
    let result = bsc._bsc_decompress(ptr,block_info[0],ptr_out,block_info[1],1);
    if(result === 0){
        let out = new Uint8Array(block_info[1]);
        out.set(new Uint8Array(window.libbsc.runtime.HEAPU8.buffer,ptr_out,block_info[1]))
        window.libbsc.runtime._free(ptr);
        return out;
    }
    throw new Error("bsc_compress failed with code "+result);
}
if(!window.libbsc){
    window.libbsc = {};
}
window.libbsc.bsc_compress_u8 = bsc_compress_u8;
window.libbsc.bsc_decompress_u8 = bsc_decompress_u8;
window.libbsc.create_bsc = create_bsc;
