
export function serialize_db(a : Uint8Array) : string {
  return window.fflate.encode_base85(window.libbsc.bsc_compress_u8(a));
}
export function deserialize_db(s : string) : Uint8Array {
  return window.libbsc.bsc_decompress_u8(window.fflate.decode_base85(s));
}
export async function load_db(sqlite_url : string | undefined = undefined){
    let el = document.getElementById("serialized_db");
    let localdb = await load_db_from_localstorage();
    if (localdb){
        return localdb;
    }
    let response : Response;
    if(!el){
        console.debug("Trying to load database from server file");
        if(!sqlite_url){
            sqlite_url = "db.sqlite"
        }
        response = await fetch(sqlite_url);
        if (!response.ok){
            throw response.statusText
        }
        let dup = await response.arrayBuffer();
        let sp = document.createElement("span");
        sp.style.display = "none";
        sp.id = "serialized_db";
        sp.textContent = serialize_db(new Uint8Array(dup));
        document.body.appendChild(sp);
        console.log("wrote db to html content");
        return dup;
    }else{
        console.debug("Trying to load database from HTML element");
        let base85 = el.textContent?.trim();
        let u8 : Uint8Array = deserialize_db(base85);
        return u8.buffer;
    }
}

export async function store_db_to_localstorage(sql_promiser : (a:any)=>Promise<{result:{byteArray:ArrayBuffer}}>){
    if(!window.localStorage){
      return false;
    }
    if(location.protocol.startsWith("http")){
      return false;
    }
    let barray = (
      await sql_promiser({
        type: "export",
      })
    ).result.byteArray;
    if(typeof CompressionStream === "function"){
      barray = await compressArrayBuffer(barray);
      window.localStorage.setItem("db_c","t");
    };
    let result = arrayBufferToString(barray);
    if (result.end_padding > 0){
      window.localStorage.setItem("db_p",result.end_padding.toString());
    }else{
      window.localStorage.removeItem("db_p");
    }
    window.localStorage.setItem("db",result.array);
    console.debug("Wrote db to localStorage");
    return true;
  }
  async function load_db_from_localstorage(){
    if(!window.localStorage){
      return false;
    }
    //TODO: !important there's no easy way to 
    //deal with conflicting changes
    //(moreso with such limited storage); for now
    //we disallow localStorage on servers, as
    //to always show the uploaded changes
    if(location.protocol.startsWith("http")){
      console.log("Ignoring localstorage");
      return false;
    }
    let binary_string = window.localStorage.getItem("db");
    if(!binary_string){
      console.log("localstorage has no stored database");
      return false;
    }
    console.debug("Trying to load database from localStorage");
    let padding = parseInt(window.localStorage.getItem("db_p") || "0");
    let array_buffer = string_to_arraybuffer(binary_string, padding);
    if(window.localStorage.getItem("db_c")){
      array_buffer = await decompressArrayBuffer(array_buffer);
    }
    return array_buffer;
}
  function string_to_arraybuffer(string : string,padding: number){
    let buffer = new ArrayBuffer(string.length*2-padding);
    let array = new Uint16Array(buffer,0,string.length-(Math.ceil(padding/2)));
    for(let i = 0;i<array.length;i++){
      array[i] = string.charCodeAt(i);
    }
    if(padding & 1){
      let last = new Uint8Array(buffer,buffer.byteLength-1,1);
      last[0] = string.charCodeAt(string.length-(Math.ceil(padding/2))) % 256;
    }
    return buffer;
  }
  function compressArrayBuffer(input : ArrayBuffer) {
    const stream = new Response(input).body
      .pipeThrough(new CompressionStream('gzip'));
    return new Response(stream).arrayBuffer();
  }
  function decompressArrayBuffer(input : ArrayBuffer){
    const stream = new Response(input).body
      .pipeThrough(new DecompressionStream('gzip'));
    return new Response(stream).arrayBuffer();
  }


  function arrayBufferToString(buffer){
    let bufView;
    let is_odd = buffer.byteLength % 2 === 1;
    if (is_odd){
      bufView = new Uint16Array(buffer.slice(0,-1))
    }else{
      bufView = new Uint16Array(buffer);
    }
    let length = bufView.length;
    let result = '';
    let stack_size = Math.pow(2,16)-1;
    for(var i = 0;i<length;i+=stack_size){

        if(i + stack_size > length){
          stack_size = length - i;
        }
        result += String.fromCharCode.apply(null, bufView.subarray(i,i+stack_size));
    }
    if (is_odd){
      let byte = buffer.slice(-1);
      result += String.fromCharCode(byte)
    }
    return {
      array: result,
      end_padding: is_odd? 1: 0
    };
  }