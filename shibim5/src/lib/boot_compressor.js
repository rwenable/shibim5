addEventListener("DOMContentLoaded", () => {
  document.getElementById("boot-msg").textContent = "Cargando...";
});
addEventListener("load",async ()=>{
  console.log("Decompressing libbsc");
  let bsc_el = document.getElementById("bsc-wasm");
  let bsc_u8 = window.fflate.decode_base85(bsc_el.textContent.trim());
  let bsc_ab = window.fflate.unzlibSync(bsc_u8).buffer;
  window.libbsc.runtime = await window.libbsc.module({wasmBinary : bsc_ab});
  if(libbsc.runtime._bsc_init(1) !== 0){
      console.error("Could not init libbsc");
  }
  console.log("Decompressing content");
  let enc = document.getElementById("__enc__");
  let dec = window.fflate.decode_base85(enc.textContent.trim());
  let u8 = window.libbsc.bsc_decompress_u8(dec);
  let el = document.createElement("script");
  el.setAttribute("type","module");
  el.textContent = window.fflate.strFromU8(u8);
  let senc = document.getElementById("__sty__");
  let sdec = window.fflate.decode_base85(senc.textContent.trim());
  let su8 = window.libbsc.bsc_decompress_u8(sdec);
  let sel = document.createElement("style");
  sel.textContent = window.fflate.strFromU8(su8);
  console.log("Booting up");
  document.head.appendChild(el);
  document.head.appendChild(sel);
  enc.remove();
  senc.remove();
});