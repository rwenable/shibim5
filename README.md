# Shibim5 
This is a chord chart song book software.
It works as a self contained html file;
you can upload it to a static site host,
or even use it locally without a web server.

There's rudimentary functionality to present
the lyrics in a new window.
Abcjs is included to allow some form of score writing.

However, all data is stored in the same html file (within an embedded sqlite databse),
and one must 'redownload' the file in order
to save the changes.
Also, the amount of data stored is limited
by the amount of memory available to the web page.
It applies compression to the content stored.

Currently the interface is in spanish only. Sorry.

Its internals are a mess.

If no databsae was previously stored (for instance, a new build),
the web page will try to fetch a `db.sqlite` containing the song data in the same directory.
(It may fail because of CORS)

# Building
Building in windows is broken, only because abcjs is being patched by using *nix file commands.
(Modify the "postinstall" section in package.json if you want to try).

All dependencies not in npm or that have been modified
are prebuilt. Thus, one can run `npm install` and `npm run build` in the shibim5 subdirectory to build the main application.
It will be put into the shibim5/dist/ folder.
Because of the poor hackery used to make this a self contained html,
**`npm run dev` does not work, because it doesn't inline resources**

If desired, those dependecies can also be built but they require special treatment:
- shibim-parse: these are the Rust parsers for the song and list languages. No need to build this, it's a dependency of shibim-js.
- shibim-js: exposes shibim-parse in a way that is not cumbersome to js.
- fflate_min: redistribution of fflate with select functions and a base85 encoder/decoder. The minified result is required as an include later.
- libbsc-wasm: Libbsc (by Ilya Grebnov) is a compressor written in C; it has significantly better compression ratio than fflate, but it is a bit slower. It generates a wasm module.

To build shibim-js you need the rust toolchain and wasm-pack. `wasm-pack build --target web` in the shibim-js folder.

To build fflate_min, run `npm run build` in fflate_min/fflate_min. Then, copy fflate_min/fflate_min/dist/fflate.umd.cjs to shibim5/include/fflate.umd.cjs.hbs

To build libbsc, emscripten is required (https://emscripten.org/docs/getting_started/downloads.html#installation-instructions-using-the-emsdk-recommended). Run `make` in the libbsc-wasm directory.
It should generate a .wasm and .js file.
Then, the .wasm file needs to be compressed by fflate and encoded in a variant of base 85 (using fflate_min/fflate_encode_helper), the resulting file should be stored in shibim5/include/bsc.zlib.base85.hbs
The .js file just needs to be copied as shibim5/include/bsc.js.hbs

Also note that the built html's js code is not bsc compressed until it is 'downloaded' for the first time.

# *Possible* future improvements
- Fix the messy build process
- There is a severe bug in the code that calls bsc. It limits the sqlite db size (currently) to 5 MB, but it's certainly fixable ~~(I just need to read libbsc's source code)~~
- Remove handlebars dependency (It was the fastest way to put something in the html, but it is certainly not the intended use fot that library)
- All the rust lints
- Internationalization
- Some ugly bugs that desync codemirror editor (I have not been able to reproduce them consistently)
- Figure out how to take advantage of persistent storage (OPFS, WebStorage)
- Better text presenting
- Remove svelte dependency (truly, it is only properly used to generate index pages and tabs)
- ~~Rewrite everything again (why do you think it has the number 5?)~~
