(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const o of document.querySelectorAll('link[rel="modulepreload"]'))t(o);new MutationObserver(o=>{for(const i of o)if(i.type==="childList")for(const a of i.addedNodes)a.tagName==="LINK"&&a.rel==="modulepreload"&&t(a)}).observe(document,{childList:!0,subtree:!0});function n(o){const i={};return o.integrity&&(i.integrity=o.integrity),o.referrerPolicy&&(i.referrerPolicy=o.referrerPolicy),o.crossOrigin==="use-credentials"?i.credentials="include":o.crossOrigin==="anonymous"?i.credentials="omit":i.credentials="same-origin",i}function t(o){if(o.ep)return;o.ep=!0;const i=n(o);fetch(o.href,i)}})();var p=Uint8Array,G=Uint16Array,pr=Int32Array,lr=new p([0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2,3,3,3,3,4,4,4,4,5,5,5,5,0,0,0,0]),cr=new p([0,0,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13,0,0]),gr=new p([16,17,18,0,8,7,9,6,10,5,11,4,12,3,13,2,14,1,15]),Mr=function(r,e){for(var n=new G(31),t=0;t<31;++t)n[t]=e+=1<<r[t-1];for(var o=new pr(n[30]),t=1;t<30;++t)for(var i=n[t];i<n[t+1];++i)o[i]=i-n[t]<<5|t;return{b:n,r:o}},Or=Mr(lr,2),Rr=Or.b,wr=Or.r;Rr[28]=258,wr[258]=28;var Ur=Mr(cr,0),Br=Ur.b,kr=Ur.r,yr=new G(32768);for(var w=0;w<32768;++w){var Q=(w&43690)>>1|(w&21845)<<1;Q=(Q&52428)>>2|(Q&13107)<<2,Q=(Q&61680)>>4|(Q&3855)<<4,yr[w]=((Q&65280)>>8|(Q&255)<<8)>>1}var j=function(r,e,n){for(var t=r.length,o=0,i=new G(e);o<t;++o)r[o]&&++i[r[o]-1];var a=new G(e);for(o=1;o<e;++o)a[o]=a[o-1]+i[o-1]<<1;var c;if(n){c=new G(1<<e);var v=15-e;for(o=0;o<t;++o)if(r[o])for(var d=o<<4|r[o],f=e-r[o],l=a[r[o]-1]++<<f,h=l|(1<<f)-1;l<=h;++l)c[yr[l]>>v]=d}else for(c=new G(t),o=0;o<t;++o)r[o]&&(c[o]=yr[a[r[o]-1]++]>>15-r[o]);return c},J=new p(288);for(var w=0;w<144;++w)J[w]=8;for(var w=144;w<256;++w)J[w]=9;for(var w=256;w<280;++w)J[w]=7;for(var w=280;w<288;++w)J[w]=8;var ir=new p(32);for(var w=0;w<32;++w)ir[w]=5;var Dr=j(J,9,0),Fr=j(J,9,1),Tr=j(ir,5,0),Yr=j(ir,5,1),sr=function(r){for(var e=r[0],n=1;n<r.length;++n)r[n]>e&&(e=r[n]);return e},T=function(r,e,n){var t=e/8|0;return(r[t]|r[t+1]<<8)>>(e&7)&n},hr=function(r,e){var n=e/8|0;return(r[n]|r[n+1]<<8|r[n+2]<<16)>>(e&7)},Cr=function(r){return(r+7)/8|0},ur=function(r,e,n){return(e==null||e<0)&&(e=0),(n==null||n>r.length)&&(n=r.length),new p(r.subarray(e,n))},jr=["unexpected EOF","invalid block type","invalid length/literal","invalid distance","stream finished","no stream handler",,"no callback","invalid UTF-8 data","extra field too long","date not in range 1980-2099","filename too long","stream finishing","invalid zip data"],B=function(r,e,n){var t=new Error(e||jr[r]);if(t.code=r,Error.captureStackTrace&&Error.captureStackTrace(t,B),!n)throw t;return t},Vr=function(r,e,n,t){var o=r.length,i=t?t.length:0;if(!o||e.f&&!e.l)return n||new p(0);var a=!n,c=a||e.i!=2,v=e.i;a&&(n=new p(o*3));var d=function(er){var nr=n.length;if(er>nr){var _=new p(Math.max(nr*2,er));_.set(n),n=_}},f=e.f||0,l=e.p||0,h=e.b||0,y=e.l,E=e.d,b=e.m,M=e.n,D=o*8;do{if(!y){f=T(r,l,1);var R=T(r,l+1,3);if(l+=3,R)if(R==1)y=Fr,E=Yr,b=9,M=5;else if(R==2){var F=T(r,l,31)+257,O=T(r,l+10,15)+4,z=F+T(r,l+5,31)+1;l+=14;for(var k=new p(z),C=new p(19),u=0;u<O;++u)C[gr[u]]=T(r,l+u*3,7);l+=O*3;for(var s=sr(C),P=(1<<s)-1,H=j(C,s,1),u=0;u<z;){var S=H[T(r,l,P)];l+=S&15;var A=S>>4;if(A<16)k[u++]=A;else{var x=0,L=0;for(A==16?(L=3+T(r,l,3),l+=2,x=k[u-1]):A==17?(L=3+T(r,l,7),l+=3):A==18&&(L=11+T(r,l,127),l+=7);L--;)k[u++]=x}}var U=k.subarray(0,F),g=k.subarray(F);b=sr(U),M=sr(g),y=j(U,b,1),E=j(g,M,1)}else B(1);else{var A=Cr(l)+4,I=r[A-4]|r[A-3]<<8,X=A+I;if(X>o){v&&B(0);break}c&&d(h+I),n.set(r.subarray(A,X),h),e.b=h+=I,e.p=l=X*8,e.f=f;continue}if(l>D){v&&B(0);break}}c&&d(h+131072);for(var rr=(1<<b)-1,$=(1<<M)-1,V=l;;V=l){var x=y[hr(r,l)&rr],K=x>>4;if(l+=x&15,l>D){v&&B(0);break}if(x||B(2),K<256)n[h++]=K;else if(K==256){V=l,y=null;break}else{var N=K-254;if(K>264){var u=K-257,m=lr[u];N=T(r,l,(1<<m)-1)+Rr[u],l+=m}var Y=E[hr(r,l)&$],W=Y>>4;Y||B(3),l+=Y&15;var g=Br[W];if(W>3){var m=cr[W];g+=hr(r,l)&(1<<m)-1,l+=m}if(l>D){v&&B(0);break}c&&d(h+131072);var Z=h+N;if(h<g){var ar=i-g,fr=Math.min(g,Z);for(ar+h<0&&B(3);h<fr;++h)n[h]=t[ar+h]}for(;h<Z;++h)n[h]=n[h-g]}}e.l=y,e.p=V,e.b=h,e.f=f,y&&(f=1,e.m=b,e.d=E,e.n=M)}while(!f);return h!=n.length&&a?ur(n,0,h):n.subarray(0,h)},q=function(r,e,n){n<<=e&7;var t=e/8|0;r[t]|=n,r[t+1]|=n>>8},tr=function(r,e,n){n<<=e&7;var t=e/8|0;r[t]|=n,r[t+1]|=n>>8,r[t+2]|=n>>16},vr=function(r,e){for(var n=[],t=0;t<r.length;++t)r[t]&&n.push({s:t,f:r[t]});var o=n.length,i=n.slice();if(!o)return{t:Nr,l:0};if(o==1){var a=new p(n[0].s+1);return a[n[0].s]=1,{t:a,l:1}}n.sort(function(z,k){return z.f-k.f}),n.push({s:-1,f:25001});var c=n[0],v=n[1],d=0,f=1,l=2;for(n[0]={s:-1,f:c.f+v.f,l:c,r:v};f!=o-1;)c=n[n[d].f<n[l].f?d++:l++],v=n[d!=f&&n[d].f<n[l].f?d++:l++],n[f++]={s:-1,f:c.f+v.f,l:c,r:v};for(var h=i[0].s,t=1;t<o;++t)i[t].s>h&&(h=i[t].s);var y=new G(h+1),E=br(n[f-1],y,0);if(E>e){var t=0,b=0,M=E-e,D=1<<M;for(i.sort(function(k,C){return y[C.s]-y[k.s]||k.f-C.f});t<o;++t){var R=i[t].s;if(y[R]>e)b+=D-(1<<E-y[R]),y[R]=e;else break}for(b>>=M;b>0;){var F=i[t].s;y[F]<e?b-=1<<e-y[F]++-1:++t}for(;t>=0&&b;--t){var O=i[t].s;y[O]==e&&(--y[O],++b)}E=e}return{t:new p(y),l:E}},br=function(r,e,n){return r.s==-1?Math.max(br(r.l,e,n+1),br(r.r,e,n+1)):e[r.s]=n},Ar=function(r){for(var e=r.length;e&&!r[--e];);for(var n=new G(++e),t=0,o=r[0],i=1,a=function(v){n[t++]=v},c=1;c<=e;++c)if(r[c]==o&&c!=e)++i;else{if(!o&&i>2){for(;i>138;i-=138)a(32754);i>2&&(a(i>10?i-11<<5|28690:i-3<<5|12305),i=0)}else if(i>3){for(a(o),--i;i>6;i-=6)a(8304);i>2&&(a(i-3<<5|8208),i=0)}for(;i--;)a(o);i=1,o=r[c]}return{c:n.subarray(0,t),n:e}},or=function(r,e){for(var n=0,t=0;t<e.length;++t)n+=r[t]*e[t];return n},Kr=function(r,e,n){var t=n.length,o=Cr(e+2);r[o]=t&255,r[o+1]=t>>8,r[o+2]=r[o]^255,r[o+3]=r[o+1]^255;for(var i=0;i<t;++i)r[o+i+4]=n[i];return(o+4+t)*8},Er=function(r,e,n,t,o,i,a,c,v,d,f){q(e,f++,n),++o[256];for(var l=vr(o,15),h=l.t,y=l.l,E=vr(i,15),b=E.t,M=E.l,D=Ar(h),R=D.c,F=D.n,O=Ar(b),z=O.c,k=O.n,C=new G(19),u=0;u<R.length;++u)++C[R[u]&31];for(var u=0;u<z.length;++u)++C[z[u]&31];for(var s=vr(C,7),P=s.t,H=s.l,S=19;S>4&&!P[gr[S-1]];--S);var A=d+5<<3,x=or(o,J)+or(i,ir)+a,L=or(o,h)+or(i,b)+a+14+3*S+or(C,P)+2*C[16]+3*C[17]+7*C[18];if(v>=0&&A<=x&&A<=L)return Kr(e,f,r.subarray(v,v+d));var U,g,I,X;if(q(e,f,1+(L<x)),f+=2,L<x){U=j(h,y,0),g=h,I=j(b,M,0),X=b;var rr=j(P,H,0);q(e,f,F-257),q(e,f+5,k-1),q(e,f+10,S-4),f+=14;for(var u=0;u<S;++u)q(e,f+3*u,P[gr[u]]);f+=3*S;for(var $=[R,z],V=0;V<2;++V)for(var K=$[V],u=0;u<K.length;++u){var N=K[u]&31;q(e,f,rr[N]),f+=P[N],N>15&&(q(e,f,K[u]>>5&127),f+=K[u]>>12)}}else U=Dr,g=J,I=Tr,X=ir;for(var u=0;u<c;++u){var m=t[u];if(m>255){var N=m>>18&31;tr(e,f,U[N+257]),f+=g[N+257],N>7&&(q(e,f,m>>23&31),f+=lr[N]);var Y=m&31;tr(e,f,I[Y]),f+=X[Y],Y>3&&(tr(e,f,m>>5&8191),f+=cr[Y])}else tr(e,f,U[m]),f+=g[m]}return tr(e,f,U[256]),f+g[256]},qr=new pr([65540,131080,131088,131104,262176,1048704,1048832,2114560,2117632]),Nr=new p(0),Hr=function(r,e,n,t,o,i){var a=i.z||r.length,c=new p(t+a+5*(1+Math.ceil(a/7e3))+o),v=c.subarray(t,c.length-o),d=i.l,f=(i.r||0)&7;if(e){f&&(v[0]=i.r>>3);for(var l=qr[e-1],h=l>>13,y=l&8191,E=(1<<n)-1,b=i.p||new G(32768),M=i.h||new G(E+1),D=Math.ceil(n/3),R=2*D,F=function(dr){return(r[dr]^r[dr+1]<<D^r[dr+2]<<R)&E},O=new pr(25e3),z=new G(288),k=new G(32),C=0,u=0,s=i.i||0,P=0,H=i.w||0,S=0;s+2<a;++s){var A=F(s),x=s&32767,L=M[A];if(b[x]=L,M[A]=x,H<=s){var U=a-s;if((C>7e3||P>24576)&&(U>423||!d)){f=Er(r,v,0,O,z,k,u,P,S,s-S,f),P=C=u=0,S=s;for(var g=0;g<286;++g)z[g]=0;for(var g=0;g<30;++g)k[g]=0}var I=2,X=0,rr=y,$=x-L&32767;if(U>2&&A==F(s-$))for(var V=Math.min(h,U)-1,K=Math.min(32767,s),N=Math.min(258,U);$<=K&&--rr&&x!=L;){if(r[s+I]==r[s+I-$]){for(var m=0;m<N&&r[s+m]==r[s+m-$];++m);if(m>I){if(I=m,X=$,m>V)break;for(var Y=Math.min($,m-2),W=0,g=0;g<Y;++g){var Z=s-$+g&32767,ar=b[Z],fr=Z-ar&32767;fr>W&&(W=fr,L=Z)}}}x=L,L=b[x],$+=x-L&32767}if(X){O[P++]=268435456|wr[I]<<18|kr[X];var er=wr[I]&31,nr=kr[X]&31;u+=lr[er]+cr[nr],++z[257+er],++k[nr],H=s+I,++C}else O[P++]=r[s],++z[r[s]]}}for(s=Math.max(s,H);s<a;++s)O[P++]=r[s],++z[r[s]];f=Er(r,v,d,O,z,k,u,P,S,s-S,f),d||(i.r=f&7|v[f/8|0]<<3,f-=7,i.h=M,i.p=b,i.i=s,i.w=H)}else{for(var s=i.w||0;s<a+d;s+=65535){var _=s+65535;_>=a&&(v[f/8|0]=d,_=a),f=Kr(v,f+1,r.subarray(s,_))}i.i=a}return ur(c,0,t+Cr(f)+o)},Gr=function(){var r=1,e=0;return{p:function(n){for(var t=r,o=e,i=n.length|0,a=0;a!=i;){for(var c=Math.min(a+2655,i);a<c;++a)o+=t+=n[a];t=(t&65535)+15*(t>>16),o=(o&65535)+15*(o>>16)}r=t,e=o},d:function(){return r%=65521,e%=65521,(r&255)<<24|(r&65280)<<8|(e&255)<<8|e>>8}}},Qr=function(r,e,n,t,o){if(!o&&(o={l:1},e.dictionary)){var i=e.dictionary.subarray(-32768),a=new p(i.length+r.length);a.set(i),a.set(r,i.length),r=a,o.w=i.length}return Hr(r,e.level==null?6:e.level,e.mem==null?Math.ceil(Math.max(8,Math.min(13,Math.log(r.length)))*1.5):12+e.mem,n,t,o)},Xr=function(r,e,n){for(;n;++e)r[e]=n,n>>>=8},Jr=function(r,e){var n=e.level,t=n==0?0:n<6?1:n==9?3:2;if(r[0]=120,r[1]=t<<6|(e.dictionary&&32),r[1]|=31-(r[0]<<8|r[1])%31,e.dictionary){var o=Gr();o.p(e.dictionary),Xr(r,2,o.d())}},Wr=function(r,e){return((r[0]&15)!=8||r[0]>>4>7||(r[0]<<8|r[1])%31)&&B(6,"invalid zlib data"),(r[1]>>5&1)==+!e&&B(6,"invalid zlib data: "+(r[1]&32?"need":"unexpected")+" dictionary"),(r[1]>>3&4)+2};function Sr(r,e){e||(e={});var n=Gr();n.p(r);var t=Qr(r,e,e.dictionary?6:2,4);return Jr(t,e),Xr(t,t.length-4,n.d()),t}function Zr(r,e){return Vr(r.subarray(Wr(r,e&&e.dictionary),-4),{i:2},e&&e.out,e&&e.dictionary)}var Lr=typeof TextEncoder<"u"&&new TextEncoder,mr=typeof TextDecoder<"u"&&new TextDecoder,_r=0;try{mr.decode(Nr,{stream:!0}),_r=1}catch{}var re=function(r){for(var e="",n=0;;){var t=r[n++],o=(t>127)+(t>223)+(t>239);if(n+o>r.length)return{s:e,r:ur(r,n-1)};o?o==3?(t=((t&15)<<18|(r[n++]&63)<<12|(r[n++]&63)<<6|r[n++]&63)-65536,e+=String.fromCharCode(55296|t>>10,56320|t&1023)):o&1?e+=String.fromCharCode((t&31)<<6|r[n++]&63):e+=String.fromCharCode((t&15)<<12|(r[n++]&63)<<6|r[n++]&63):e+=String.fromCharCode(t)}};function ee(r,e){if(e){for(var n=new p(r.length),t=0;t<r.length;++t)n[t]=r.charCodeAt(t);return n}if(Lr)return Lr.encode(r);for(var o=r.length,i=new p(r.length+(r.length>>1)),a=0,c=function(l){i[a++]=l},t=0;t<o;++t){if(a+5>i.length){var v=new p(a+8+(o-t<<1));v.set(i),i=v}var d=r.charCodeAt(t);d<128||e?c(d):d<2048?(c(192|d>>6),c(128|d&63)):d>55295&&d<57344?(d=65536+(d&1047552)|r.charCodeAt(++t)&1023,c(240|d>>18),c(128|d>>12&63),c(128|d>>6&63),c(128|d&63)):(c(224|d>>12),c(128|d>>6&63),c(128|d&63))}return ur(i,0,a)}function ne(r,e){if(e){for(var n="",t=0;t<r.length;t+=16384)n+=String.fromCharCode.apply(null,r.subarray(t,t+16384));return n}else{if(mr)return mr.decode(r);var o=re(r),i=o.s,n=o.r;return n.length&&B(8),i}}const $r="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?`'~()[]|}@%$#",te=new Uint8Array([...$r].map(r=>r.charCodeAt(0)));let xr=new Uint8Array(256);xr.fill(255);te.forEach((r,e)=>xr[r]=e);const oe=[1,85,85*85,85*85*85,85*85*85*85];function zr(r){let e="";return r.forEach(n=>{for(let t=0;t<5;t++){let o=n%85;e+=$r.charAt(o),n=(n-o)/85}}),e}function ie(r){if(r>=142085830)throw new Error("Encoding > 142 MB not supported (nor recommended)")}function Pr(r,e){if((r.length-e)%5!==0)throw new Error("Invalid base85 encoding; character count % 5 != 0");let n=r.length/5,t=new Uint32Array(n);for(let o=e;o<n;o++){let i=0;for(let a=0;a<5;a++)i+=oe[a]*xr[r.charCodeAt(o*5+a)];t[o]=i}return t}function Ir(r){let e=r.byteLength-r.byteLength%4,n=r.byteLength%4,t=(4-n)%4,o=new Uint32Array(r,0,e/4),i=ie(r.byteLength)+zr(o);if(t){let a=0,c=new Uint8Array(r,e,r.byteLength%4);for(let v=0;v<n;v++)a+=c[v]<<v*8;i+=zr(new Uint32Array([a]))}return i}function ae(r){let e=Pr(r,5),n=Pr(r.slice(0,5),0);if(n[0]<4294967296)throw new Error("Wrong header format");let t=n[0]-4294967296,o=(r.length/5-1)*4-t;return(o<0||o>3)&&(console.error("Header length mismatch"),t=(r.length/5-1)*4),new Uint8Array(e.buffer,0,t)}document.getElementById("file").addEventListener("change",async r=>{window.ff={zlibSync:Sr,unzlibSync:Zr,strToU8:ee,strFromU8:ne,encode_base85:Ir,decode_base85:ae};let e=r.target.files;if(!e[0])return;let t=await e[0].arrayBuffer(),o=Ir(Sr(new Uint8Array(t)).buffer);document.getElementById("app").textContent=o.encoded,document.getElementById("pad").textContent=o.padding;let i=new Blob([o.encoded],{type:"text/plain"}),a=document.createElement("a");a.download="encoded.base85",a.href=window.URL.createObjectURL(i),document.body.appendChild(a),a.click(),document.body.removeChild(a)});
