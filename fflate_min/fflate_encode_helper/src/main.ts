import {zlibSync, unzlibSync, strToU8, strFromU8, encode_base85, decode_base85} from "fflate_min"
 window.ff = {zlibSync, unzlibSync, strToU8, strFromU8, encode_base85, decode_base85};
document.getElementById("file").addEventListener("change",async (e)=>{
    let files = e.target.files;
    if (!files[0]){
      return;
    }
    let file = files[0];
    let buffer = await file.arrayBuffer();
    let enc = encode_base85(zlibSync(new Uint8Array(buffer)));
    document.getElementById("app").textContent = enc;
    let blob = new Blob([enc], { type: "text/plain" });
    let link = document.createElement("a");
    link.download = "encoded.base85";
    link.href = window.URL.createObjectURL(blob);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
});
