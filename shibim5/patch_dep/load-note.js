// Load one mp3 file for one note.
// url = the base url for the soundfont
// instrument = the instrument name (e.g. "acoustic_grand_piano")
// name = the pitch name (e.g. "A3")
var soundsCache = require("./sounds-cache");
const NOTE_HEIGHTS = {
	"C" : 0,
	"D" : 2,
	"E" : 4,
	"F" : 5,
	"G" : 7,
	"A" : 9,
	"B" : 11
};
//https://kokkinizita.linuxaudio.org/papers/digsvfilt.pdf
var sv_filter = function(arr_in, arr_out, q,fc,sr){
	let w = 2*Math.tan(fc*Math.PI/sr);
	let a =  w/q;
	let b = w*w;
	let c1 = (a+b)/(1+a/2+b/4);
	let c2 = b/(b+a);
	let d0 = (1-c2)*c1/2;
	let d1 = (1-c2);
	let z2 = 0, z1 = 0;
	for(let i = 0;i<arr_in.length;i++){
		let x = arr_in[i]-z1-z2;
		arr_out[i] = d0*x + d1*z1;
		z2 += c2 * z1;
		z1 += c1 * x;
	}
}
var normalize_distort_arr = function(arr_inout,max){
	let amax = 0;
	for(let i = 0;i < arr_inout.length; i++){
		amax = Math.max(Math.abs(arr_inout[i]),amax)
	}
	let factor = max/amax;
	for(let i = 0;i < arr_inout.length; i++){
		arr_inout[i] = factor*arr_inout[i];
		//arr_inout[i] = factor*(1.5*arr_inout[i] - arr_inout[i]*arr_inout[i]/2/amax);
	}
}
const ROOT_12 = Math.pow(2,1/12);
var decodeNote = function(name){
	let letter = name[0];
	let accidental;
	let octave_str;
	if (name[1] == "#"){
		accidental = 1;
		octave_str = name.slice(2);
	}else if(name[1] == "b"){
		accidental = -1;
		octave_str = name.slice(2);
	}else{
		accidental = 0;
		octave_str = name.slice(1);
	}
	let octave = parseInt(octave_str);
	let note_h = NOTE_HEIGHTS[letter];
		//A0 = 27.5;
	let total_h = octave*12+note_h - NOTE_HEIGHTS["A"] + accidental;
	let freq = 27.5*Math.pow(2,total_h/12);
	return freq;
}
//Noise with sharp decay and slight low-pass
var createClick = function (audioContext, sample_rate, freq = 220){
	let amp = 1;
	let decay_target = -0.3;
	let decay_k = 0.99965;
	let len = Math.ceil(sample_rate*0.15);
	let buffer = audioContext.createBuffer(1,len,sample_rate);
	let out_buffer = audioContext.createBuffer(1,len,sample_rate);
	let array = buffer.getChannelData(0);
	for (let i = 0; i < buffer.length;i++){
		array[i] = amp*(Math.random()*2-1);
		amp = amp*decay_k + decay_target*(1-decay_k);
		amp = amp > 0? amp : 0;
	}
	let out_array = out_buffer.getChannelData(0);
	sv_filter(array,out_array,20,freq,sample_rate)
	normalize_distort_arr(out_array,0.1);
	return out_buffer;
}
var createPad = function (audioContext, sample_rate, freq){
	const no_harm_start_f = 80;
	const no_harm_stop_f = 1000;
	let h_d = Math.max(no_harm_stop_f-freq-no_harm_start_f,0)/(no_harm_stop_f-no_harm_start_f);
	h_d *= h_d;
	let amp = 1;
	let decay_target = -0.3;
	let decay_k = 0.99999;
	let len = Math.ceil(sample_rate*2.5);
	let buffer = audioContext.createBuffer(1,len,sample_rate);
	let out_buffer = audioContext.createBuffer(1,len,sample_rate);
	let array = buffer.getChannelData(0);
	for (let i = 0; i < buffer.length;i++){
		array[i] = amp*(Math.random()*2-1);
		amp = amp*decay_k + decay_target*(1-decay_k);
		amp = amp > 0? amp : 0;
	}
	let out_array = out_buffer.getChannelData(0);
	let tmp_array = new Float32Array(out_array.length);
	let tmp2_array = new Float32Array(out_array.length);
	function add_harm(harm,amp){
		sv_filter(array,tmp2_array,60,freq*harm,sample_rate);
		sv_filter(tmp2_array,tmp_array,60,freq*harm,sample_rate);
		let sign = 1;
		if (amp < 0){
			amp = -amp;
			sign = -1;
		}
		normalize_distort_arr(tmp_array,amp);
		for(let i = 0; i< buffer.length;i++){
			out_array[i] += sign*tmp_array[i]
		}
	}
	sv_filter(array,tmp2_array,200,freq,sample_rate);
	sv_filter(tmp2_array,out_array,200,freq,sample_rate);

	normalize_distort_arr(out_array,1);
	add_harm(2,-0.7*h_d);
	add_harm(3,0.5*h_d*h_d*h_d);
	add_harm(4,-0.7*h_d*h_d);
	add_harm(5,0.3*h_d*h_d*h_d);
	add_harm(6,-0.5*h_d*h_d*h_d);
	add_harm(8,0.5*h_d*h_d*h_d*h_d);
	add_harm(9,0.2*h_d*h_d*h_d*h_d);
	normalize_distort_arr(out_array,0.05*(h_d+1));
	return out_buffer;
}
function calc_decay_constants(k_curv,d_time){
	return {
		k : Math.pow(k_curv,1/d_time),
		a : (k_curv-Math.pow(k_curv,(d_time+1)/d_time))/(k_curv-1)
	}
}
//Ugly additive synth
var createSine = function (audioContext, sample_rate, freq){
	let amp = 1;
	let ramp = 48;
	let decay_target = -0.3;
	let decay_k = 0.99999;
	let no_harm_point = 4000;
	let bass_point = 220;
	let duration = Math.max(3 - 2/400*(freq-100),0.6)*sample_rate;
	let dks = calc_decay_constants(0.05,duration);
	console.log(duration/sample_rate,dks);
	let at = Math.max(no_harm_point - freq -bass_point,0)/(no_harm_point-bass_point);
	let at_b = Math.max(bass_point - freq,0)/bass_point
	let buffer = audioContext.createBuffer(1,duration,sample_rate);
	let array = buffer.getChannelData(0);
	let wv = Math.min(freq/1000,1);
	let phase = ((freq*freq)%(2*Math.PI));
	console.log(at);
	let mfun = function(x){
		let s = Math.sin(x + phase);
		let s3 = s*s*s;
		let bass = Math.cos(s*4.75)*at_b*at_b;
		return bass + s + at*s3 - at*at*s3*s*s + at*at*at*(s3*s-0.5) + at*at*at*at*(s3*s3-0.5) + at*at*at*at*at*(s3*s3*s3-21/32*s3)/2;
	};
	for (let i = 0; i < buffer.length;i++){
		let ii = (i*(1+0.0003*Math.sin(i*(2*wv-wv*wv)/2600 + freq)))
		if(i < ramp){
			array[i] = amp*(ii/ramp)*mfun(ii*2*Math.PI/sample_rate*freq);
		}else{
			array[i] = amp*mfun(ii*2*Math.PI/sample_rate*freq);
			amp = amp*dks.k + dks.a;
			amp = amp > 0? amp : 0;
		}
	}
	normalize_distort_arr(array,0.085);
	return buffer;
}

var getNote = function (_url, instrument, name, audioContext) {
	if (!soundsCache[instrument]) soundsCache[instrument] = {};
	var instrumentCache = soundsCache[instrument];

	if (!instrumentCache[name]){
		console.log(`ins:${instrument} name:${name}`)
		instrumentCache[name] = new Promise(function (resolve, reject) {
			if(instrument === "percussion"){
				resolve({instrument: instrument, name: name, status: "loaded", audioBuffer: createClick(audioContext,48000,decodeNote(name))})
			}else if(instrument === "reed_organ"){
				resolve({instrument: instrument, name: name, status: "loaded", audioBuffer: createPad(audioContext,48000,decodeNote(name))})
			}else{
				resolve({instrument: instrument, name: name, status: "loaded", audioBuffer: createSine(audioContext,48000,decodeNote(name))})
			}
		})
	}
	return instrumentCache[name];
};


var old_getNote = function (url, instrument, name, audioContext) {
	console.log("HELLO FROM GETNOTE")
	if (!soundsCache[instrument]) soundsCache[instrument] = {};
	var instrumentCache = soundsCache[instrument];

	if (!instrumentCache[name])
		instrumentCache[name] = new Promise(function (resolve, reject) {
			var xhr = new XMLHttpRequest();
			let noteUrl = url + instrument + "-mp3/" + name + ".mp3";
			xhr.open("GET", noteUrl, true);
			xhr.responseType = "arraybuffer";
			xhr.onload = function () {
				if (xhr.status !== 200) {
					reject(Error("Can't load sound at " + noteUrl + ' status=' + xhr.status));
					return
				}
				var noteDecoded = function(audioBuffer) {
					resolve({instrument: instrument, name: name, status: "loaded", audioBuffer: audioBuffer})
				}
				var maybePromise = audioContext.decodeAudioData(xhr.response, noteDecoded, function () {
					reject(Error("Can't decode sound at " + noteUrl));
				});
				// In older browsers `BaseAudioContext.decodeAudio()` did not return a promise
				if (maybePromise && typeof maybePromise.catch === "function") maybePromise.catch(reject);
			};
			xhr.onerror = function () {
				reject(Error("Can't load sound at " + noteUrl));
			};
			xhr.send();
		})
			.catch(err => {
				console.error("Didn't load note", instrument, name, ":", err.message);
				throw err;
			});

	return instrumentCache[name];
};

module.exports = getNote;
