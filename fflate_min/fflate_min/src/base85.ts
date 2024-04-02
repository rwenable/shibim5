const base85_str =
"0123456789"+
"abcdefghij"+
"klmnopqrst"+
"uvwxyzABCD"+
"EFGHIJKLMN"+
"OPQRSTUVWX"+
"YZ.-:+=^!/"+
"*?`'~()[]|"+
"}@%$#";
const past_max_int = 4294967296;
const base85_lu = new Uint8Array([...base85_str].map((e)=>e.charCodeAt(0)));
let base64_rlu = new Uint8Array(256);
base64_rlu.fill(255);
base85_lu.forEach((n,i)=>base64_rlu[n]=i);
const p85 = [1,85,85*85,85*85*85,85*85*85*85];

function encode_base85_ta(a : Uint32Array){
    let start = "";
    a.forEach((n)=>{
        //TODO: only 4 + remainder is necessary
        for(let d = 0;d < 5;d++){
            let r = n % 85;
            start += base85_str.charAt(r);
            n = (n-r)/85;
        }
    });
    return start;
}
function encode_header(len : number){
	if (len >= 142085830){
		throw new Error("Encoding > 142 MB not supported (nor recommended)");
	}
	let start = "";
	let n = past_max_int + len;
	for(let d = 0;d < 5;d++){
		let r = n % 85;
		start += base85_str.charAt(r);
		n = (n-r)/85;
	}
	return start;
}
function decode_base85_ta(s : string, offset : number){
    if ((s.length - offset) % 5 !== 0){
        throw new Error("Invalid base85 encoding; character count % 5 != 0");
    }
    let n_blocks = (s.length - offset) / 5;
    let output = new Uint32Array(n_blocks);
    for(let i=0;i< n_blocks;i++){
        let value = 0;
        for(let k=0;k < 5;k++){
            value += p85[k]*base64_rlu[s.charCodeAt(i*5+k+offset)]
        }
        output[i] = value;
    }
    return output;
}
export function encode_base85(a : Uint8Array){
    let main_size = a.length - (a.length % 4);
    let remainder = a.length % 4;
    let padding =  (4 - (remainder)) % 4;
    let view = new Uint32Array(a.buffer,a.byteOffset,main_size / 4);
    let out = encode_header(a.byteLength) + encode_base85_ta(view);
    if (padding){
        let value = 0;
        for (let i=0;i<remainder;i++){
            value += a[i+main_size]<<(i*8);
        }
        out += encode_base85_ta(new Uint32Array([value]));
    }
    return out;
}

export function decode_base85(s : string){
    let array = decode_base85_ta(s,5);
	let header = decode_base85_ta(s.slice(0,5),0);
	if (header[0] > 142085830 || !s[4].match(/[%$#]/)){
		throw new Error("Wrong header format ");
	}
	let len = header[0];
	let calc_pad = (s.length/5 - 1)*4 - len;
	if (calc_pad < 0 || calc_pad > 3){
		console.error("Header length mismatch");
		len = (s.length/5 - 1)*4;
	}
	return new Uint8Array(array.buffer,0, len)
}
