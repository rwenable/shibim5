const LIBBSC_FILE_HEADER_SIZE = 4;
const LIBBSC_HEADER_SIZE = 28;
const MAX_BSC_BLOCK_SIZE = 5*1024*1024;
const MAX_BSC_BLOCK_FULL_SIZE = MAX_BSC_BLOCK_SIZE + LIBBSC_FILE_HEADER_SIZE;
function create_bsc(){
    let bsc_el = document.getElementById("bsc-wasm");
    let bsc_u8 = window.fflate.decode_base85(bsc_el.textContent.trim());
    let bsc_ab = window.fflate.unzlibSync(bsc_u8).buffer;
    return window.libbsc.module({wasmBinary : bsc_ab}).then(runtime=>{
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
    let out = new Uint8Array(u8.length + LIBBSC_FILE_HEADER_SIZE)
    let header = new Uint32Array(out.buffer,0,4);
    header[0] = u8.length;
    let processed_bytes = 0;
    let written_bytes = LIBBSC_FILE_HEADER_SIZE;
    let ptr;
    if (u8.length <= MAX_BSC_BLOCK_SIZE){
        ptr = bsc._malloc(u8.length + LIBBSC_HEADER_SIZE);
    }else{
        ptr = bsc._malloc(MAX_BSC_BLOCK_SIZE + LIBBSC_HEADER_SIZE);
    }
    while(processed_bytes < u8.length){
        let subarray_end = processed_bytes + MAX_BSC_BLOCK_SIZE;
        if (subarray_end > u8.length){
            subarray_end = u8.length
        }
        bsc.HEAPU8.set(u8.subarray(processed_bytes,subarray_end),ptr);
        let result = bsc._bsc_compress(ptr,ptr,subarray_end-processed_bytes,15,128,1,1,1);
        if (result < 0){
            throw new Error("bsc_compress failed with code "+result);
        }
        out.set(new Uint8Array(bsc.HEAPU8.buffer,ptr,result),written_bytes);
        written_bytes += result;
        processed_bytes = subarray_end;
    }
    bsc._free(ptr);
    return out.subarray(0,written_bytes);
}
function bsc_decompress_u8(u8){
    if(!window.libbsc.runtime){
        create_bsc();
    }
    let bsc = window.libbsc.runtime;
    let header = new Uint32Array(u8.buffer,0,1);
    let out_size = header[0];
    let processed_bytes = 4;
    let written_bytes = 0;

    let block_info_ptr = bsc._malloc(2*4);
    let ptr = bsc._malloc(u8.length > MAX_BSC_BLOCK_FULL_SIZE? MAX_BSC_BLOCK_FULL_SIZE : u8.length);
    let out = new Uint8Array(out_size);

    while(processed_bytes < u8.length){
        bsc.HEAPU8.set(u8.subarray(processed_bytes,processed_bytes+LIBBSC_HEADER_SIZE),ptr);
        let info_res = bsc._bsc_block_info(ptr,LIBBSC_HEADER_SIZE,block_info_ptr,block_info_ptr+4,1);
        if(info_res !== 0){
            throw new Error("bsc_block_info failed with code "+info_res);
        }
        let block_info = new Uint32Array(bsc.HEAPU8.buffer,block_info_ptr,2);
        console.log("BSC block",block_info);

        bsc.HEAPU8.set(u8.subarray(processed_bytes,processed_bytes+block_info[0]),ptr);
        let ptr_out = bsc._malloc(block_info[1]);
        let result = bsc._bsc_decompress(ptr,block_info[0],ptr_out,block_info[1],1);
        if (result < 0){
            throw new Error("bsc_decompress failed with code "+result);
        }
        out.set(new Uint8Array(bsc.HEAPU8.buffer,ptr_out,block_info[1]),written_bytes);
        written_bytes += block_info[1];
        processed_bytes += block_info[0];
        bsc._free(ptr_out);
    }
    bsc._free(ptr);
    return out;
}
if(!window.libbsc){
    window.libbsc = {};
}
window.libbsc.bsc_compress_u8 = bsc_compress_u8;
window.libbsc.bsc_decompress_u8 = bsc_decompress_u8;
window.libbsc.create_bsc = create_bsc;
