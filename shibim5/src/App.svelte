<script>
  import { onMount, setContext } from "svelte";
  import SongIndex from "./SongIndex.svelte";
  import Tabs from "./lib/Tabs.svelte";
  import ListIndex from "./ListIndex.svelte";
  import { load_wasm } from "./lib/shb";
  import { shb_to_html, lst_to_html } from "shibim-js/shibim_js";
  import ViewWrapper from "./ViewWrapper.svelte";
  import Search from "./Search.svelte";
  import Editor from "./Editor.svelte";
  import { load_sqlite } from "./lib/sqlite_loader";
  import cf_worker_str from "./lib/cf_worker.js?raw"
  import {
    load_db,
    store_db_to_localstorage,
    serialize_db,
    deserialize_db,
  } from "./database_loader";
  import Progress from "./lib/Progress.svelte";
  import "./lib/keys";
  import boot_compressor_str from "./lib/boot_compressor.js?raw";
  import bsc_helper_str from "./lib/bsc_helper.js?raw";
  import { fade } from "svelte/transition";
  let progress = 50;
  let song_index = [];
  let list_index = [];
  let recently_viewed_songs = [];
  let recently_viewed_lists = [];
  let search_results = [];
  let container;
  let viewer_open = false;
  let viewer_target = "";
  let viewer_content = "";
  let editor_component;
  let tabs_component;
  let viewer_component;
  let sql_worker;
  let sql_promiser;
  let items_per_page = 30;
  let current_song_page = 0;
  let current_list_page = 0;
  let total_songs = 0;
  let total_lists = 0;
  let database_reloaded_state = "";
  let db_modified = false;
  let tab_save_open = false;
  let tab_upload_open = false;
  let viewer_id;
  let viewer_kind;
  let max_recent = 4;
  let cf_secret = "";
  let password_error = false;
  async function update_totals() {
    let res = (
      await sql_promiser({
        type: "exec",
        args: {
          sql: "select count(*) from songs union all select count(*) from lists;",
          returnValue: "resultRows",
        },
      })
    ).result.resultRows;
    total_songs = res[0][0];
    total_lists = res[1][0];
  }
  async function exec_db(args) {
    return (
      await sql_promiser({
        type: "exec",
        args: args,
      })
    ).result.resultRows;
  }
  async function get_db_array_buffer(){
    await sql_promiser("exec", {
      sql: "VACUUM",
    });
    let response = (
      await sql_promiser({
        type: "export",
      })
    ).result;
    return response.byteArray;
  }
  async function write_db_to_html() {
    let base85 = serialize_db(new Uint8Array(await get_db_array_buffer()));
    let el = document.getElementById("serialized_db");
    if (!el) {
      el = document.createElement("span");
      el.style.display = "none";
      el.id = "serialized_db";
    }
    el.textContent = base85;
    document.body.appendChild(el);
    console.log("wrote db to html content");
  }

  async function download_db() {
    await editor_component.save_unsaved();
    let response = (
      await sql_promiser({
        type: "export",
      })
    ).result;
    let file = new Blob([response.byteArray], {
      type: "application/x-sqlite3",
    });
    let link = document.createElement("a");
    link.style.display = "none";
    link.download = "shb-" + get_date_string() + ".sqlite3";
    link.href = window.URL.createObjectURL(file);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    db_modified = false;
  }
  async function merge_db(file_evt) {
    //Have to set dbId to falsy(?) value but not undefined.
    let files = file_evt.target.files;
    if (!files || !files[0]) {
      return;
    }
    let buffer = await files[0].arrayBuffer();
    await sql_promiser({
      type: "exec",
      args: {
        sql: "ATTACH 'secondary' AS secondary;",
      },
    });
    sql_worker.addEventListener("message", async function load_listener(evt) {
      console.log("load_listener", evt);
      if (evt.data.type === "error") {
        console.error(
          "Database restoring failed (possibly corrupted data)",
          evt,
        );
        sql_worker.removeEventListener("message", load_listener);
        return;
      } else {
        sql_worker.removeEventListener("message", load_listener);
        //TODO: repeated id, but different title
        await sql_promiser("exec", {
          sql: `create temp table lists_to_merge as
 select sli.id as id, sli.name as name from secondary.lists sli where
 sli.name NOT IN (select li.name from lists li);

create temp table songs_to_merge as
select ss.id as id, ss.name as name from secondary.songs ss where
ss.name NOT IN (select s.name from songs s); 

insert into songs (id,name,title,norm_title,subtitle,source,plain,sections,tonic,tonic_kind)
select s.id,s.name,s.title,s.norm_title,s.subtitle,s.source,s.plain,s.sections,s.tonic,s.tonic_kind from songs_to_merge sm
inner join secondary.songs s on s.name = sm.name;

insert into lists (name, source) select lm.name, sli.source from
lists_to_merge lm inner join secondary.lists sli on lm.id = sli.id;

delete from lists_songs as ls where ls.list in 
(select li.id from lists_to_merge lm inner join lists li on li.name = lm.name);

insert into lists_songs (list,song,position)
select l.id, s.id, sls.position from lists_to_merge lm
inner join secondary.lists_songs sls on lm.id = sls.list
inner join secondary.lists sl on sls.list = sl.id
inner join secondary.songs ss on sls.song = ss.id
inner join lists l on sl.name = l.name
inner join songs s on ss.name = s.name;
`,
        });
      }
      await sql_promiser({
        type: "exec",
        args: {
          sql: "drop table songs_to_merge; drop table lists_to_merge; DETACH secondary;",
        },
      });
      let cevent = new CustomEvent("databaseloaded", { default: true });
      window.dispatchEvent(cevent);
      await update_index();
    });
    sql_worker.postMessage(
      { type: "load", args: { payload: buffer, schema: "secondary" } },
      [buffer],
    );
  }
  async function upload_db(evt) {
    database_reloaded_state = "";
    let files = evt.target.files;
    if (!files || !files[0]) {
      return;
    }
    let backup = await get_db_array_buffer();
    let file = files[0];
    let buffer = await file.arrayBuffer();
    await sql_promiser("close", { filename: "" });
    await sql_promiser("open", { filename: "" });
    sql_worker.addEventListener("message", db_load_listener);
    window.addEventListener("databaseloaded", async function handler() {
      window.removeEventListener("databaseloaded", handler);
      let ok_schema = await verify_schema();
      if (ok_schema) {
        database_reloaded_state = "OK";
      } else {
        database_reloaded_state = "ERROR";
        await sql_promiser("close", { filename: "" });
        await sql_promiser("open", { filename: "" });
        console.debug("Database byte size: ", backup.byteLength);
        sql_worker.addEventListener("message", db_load_listener);
        sql_worker.postMessage({ type: "load", args: { payload: backup } }, [
          backup,
        ]);
      }
    });
    sql_worker.postMessage({ type: "load", args: { payload: buffer } }, [
      buffer,
    ]);
  }
  async function db_load_listener(evt) {
    sql_worker.removeEventListener("message", db_load_listener);
    if (evt.data.type === "error" && evt.data.result.input.type === "load") {
      //TODO CheckRC
      console.error("Database restoring failed (possibly corrupted data)", evt);
      database_reloaded_state = "ERROR";
    } else if (evt.data.type === "load") {
      console.debug("Database restored.");
      let cevent = new CustomEvent("databaseloaded", { default: true });
      window.dispatchEvent(cevent);
      await update_index();
    }
    progress += 20;
  }
  setContext("db", exec_db);
  onMount(() => {
    //Remove extra markup if html was saved directly
    let previous = document.getElementById("app_hydrated");
    if (previous) {
      previous.remove();
    }
    if (!window.libbsc.runtime) {
      console.log("Decompressing libbsc");
      window.libbsc.create_bsc();
    }
    window.addEventListener("databaseloaded", () => {
      route(window.location.search);
      window.addEventListener("popstate", () => route(window.location.search));
    });
    progress += 10;
    container.id = "app_hydrated";
    //Start sqlite
    Promise.allSettled([
      load_sqlite().then((o)=>{
        progress += 30;
        console.log("Done loading sqlite binary.");
        sql_worker = o.worker;
        sql_promiser = o.promiser;
      }),
      load_db().then((ab)=>{
        progress += 30;
        console.debug("Database byte size: ", ab.byteLength);
        return ab;
      }),
      load_wasm().then(() => {
        console.log("SHB wasm loaded.");
        progress += 10;
      })
    ]).then(async (results) => {
        if(results[1].status === "rejected"){
          console.error("Can't load database binary: ", results[1].reason);
          await sql_promiser("open", { filename: "" });
          await init_db();
        }else{
          let ab = results[1].value;
          await sql_promiser("open", { filename: "" });
          sql_worker.addEventListener("message", db_load_listener);
          sql_worker.postMessage({ type: "load", args: { payload: ab } }, [ab]);
        }
      });

    /*
    window.sqlite3InitModule({
        print: console.log,
        printErr: console.error
      }).then(async function (sqlite3) {
        console.log("Done initializing. Running demo...");
        try {
          db = await init_serialized_db(sqlite3);
          test_db(db);
        } catch (e) {
          console.error("Exception:", e.message);
        }
      });
      */
    window.__SHB_API__ = {};
    window.__SHB_API__.get_song_source_by_name = get_song_source_by_name;
    window.sql_promiser = sql_promiser;
    //window.store_db_to_localstorage = store_db_to_localstorage;
    //window.load_db_from_localstorage = load_db_from_localstorage;
    //window.write_db_to_html = write_db_to_html;
  });
  async function verify_schema() {
    let tests = [
      "select id, name, source from lists where 0",
      "select list, song, position from lists_songs where 0",
      "select id, name, title, norm_title, subtitle, source, plain, sections, tonic, tonic_kind from songs where 0",
      "select name, title, subtitle, plain, songs_fts5, rank from songs_fts5 where 0",
    ];
    for (let sql of tests) {
      let err = await exec_db({
        sql,
      })
        .then(() => false)
        .catch((e) => {
          console.error("Failed to verify database schema, ", e);
          return true;
        });
      if (err) {
        return false;
      }
    }
    return true;
  }
  async function init_serialized_db() {
    /*
    let array = await (await sqlite_db_raw).arrayBuffer();
    const oo = sqlite3.oo1;
    const p = sqlite3.wasm.allocFromTypedArray(array);
    const db = new oo.DB();
    //see note on allocators (https://sqlite.org/wasm/doc/trunk/api-c-style.md#sqlite3_deserialize)
    const rc = sqlite3.capi.sqlite3_deserialize(
      db.pointer, 'main', p, array.byteLength, array.byteLength,
      sqlite3.capi.SQLITE_DESERIALIZE_FREEONCLOSE | sqlite3.capi.SQLITE_DESERIALIZE_RESIZEABLE
    );
    db.checkRc(rc);
    return db;
    */
  }
  async function update_index() {
    await update_totals();
    let song_arr = (
      await sql_promiser("exec", {
        sql: "select name, title, subtitle, tonic, tonic_kind, sections from songs limit $lim offset $off",
        bind: {
          $lim: items_per_page,
          $off: current_song_page * items_per_page,
        },
        rowMode: "object",
        returnValue: "resultRows",
      })
    ).result.resultRows;
    song_arr.forEach((element) => {
      element.sections = element.sections.split("\n");
    });

    let list_arr = (
      await sql_promiser("exec", {
        sql: "select id, name from lists order by name desc limit $lim offset $off",
        bind: {
          $lim: items_per_page,
          $off: current_list_page * items_per_page,
        },
        rowMode: "object",
        returnValue: "resultRows",
      })
    ).result.resultRows;

    for (let list of list_arr) {
      let songs = (
        await sql_promiser("exec", {
          sql: `
select s.title from lists_songs ls
join songs s on s.id = ls.song
where ls.list = $list
order by ls.position`,
          bind: { $list: list.id },
          rowMode: "array",
          returnValue: "resultRows",
        })
      ).result.resultRows;
      list.songs = songs.map((r) => r[0]);
    }
    list_index = list_arr;
    song_index = song_arr;
  }
  async function get_song_source_by_name(name) {
    let result = (
      await sql_promiser("exec", {
        sql: "select source from songs where name = $name",
        bind: { $name: name },
        returnValue: "resultRows",
      })
    ).result.resultRows;
    if (result && result.length > 0) {
      return result[0][0];
    } else {
      throw new Error("Song " + name + " not found in db.");
    }
  }
  async function get_list_source_by_name(name) {
    let result = (
      await sql_promiser("exec", {
        sql: "select source from lists where name = $name",
        bind: { $name: name },
        returnValue: "resultRows",
      })
    ).result.resultRows;
    if (result && result.length > 0) {
      return result[0][0];
    } else {
      throw new Error("List " + name + " not found in db.");
    }
  }
  async function init_db() {
    const init_query = `
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS songs (
  id INTEGER PRIMARY KEY,
  name TEXT UNIQUE NOT NULL,
  title TEXT NOT NULL,
  norm_title TEXT NOT NULL,
  subtitle TEXT,
  source TEXT NOT NULL,
  plain TEXT NOT NULL,
  sections TEXT,
  tonic INTEGER NOT NULL,
  tonic_kind INTEGER NOT NULL
) STRICT; 
CREATE TABLE IF NOT EXISTS lists (
  id INTEGER PRIMARY KEY,
  name TEXT UNIQUE NOT NULL,
  source TEXT NOT NULL
) STRICT;
CREATE TABLE IF NOT EXISTS lists_songs (
  list INTEGER,
  song INTEGER,
  position INTEGER,
  CONSTRAINT pk PRIMARY KEY(list, song),
  CONSTRAINT fk_list FOREIGN KEY (list) REFERENCES lists(id) ON DELETE CASCADE,
  CONSTRAINT fk_song FOREIGN KEY (song) REFERENCES songs(id) ON DELETE CASCADE
) STRICT;
CREATE TABLE IF NOT EXISTS unsaved_buffers (
  id INTEGER PRIMARY KEY,
  content TEXT
) STRICT;
CREATE INDEX IF NOT EXISTS songs_norm_title ON songs (norm_title);
CREATE VIRTUAL TABLE IF NOT EXISTS songs_fts5 USING fts5(
  name,
  title,
  subtitle,
  plain,
  content = songs,
  content_rowid = id
);
CREATE TRIGGER IF NOT EXISTS songs_ai
AFTER
INSERT ON songs BEGIN
INSERT INTO songs_fts5(rowid, name, title, subtitle, plain)
VALUES (
    new.id,
    new.name,
    new.norm_title,
    new.subtitle,
    new.plain
  );
END;
CREATE TRIGGER IF NOT EXISTS songs_ad
AFTER DELETE ON songs BEGIN
INSERT INTO songs_fts5(songs_fts5, rowid, name, title, subtitle, plain)
VALUES (
    'delete',
    old.id,
    old.name,
    old.norm_title,
    old.subtitle,
    old.plain
  );
END;
CREATE TRIGGER IF NOT EXISTS songs_ud
AFTER
UPDATE ON songs BEGIN
INSERT INTO songs_fts5(songs_fts5, rowid, name, title, subtitle, plain)
VALUES (
    'delete',
    old.id,
    old.name,
    old.norm_title,
    old.subtitle,
    old.plain
  );
INSERT INTO songs_fts5(rowid, name, title, subtitle, plain)
VALUES (
    new.id,
    new.name,
    new.norm_title,
    new.subtitle,
    new.plain
  );
END;`;
    return await sql_promiser("exec", {
      sql: init_query,
    });
  }
  async function ft_search(query) {
    const ft_query = `SELECT s.rowid AS id,
    s.name AS handle,
    z.title AS title,
    s.subtitle AS subtitle,
    z.tonic AS tonic,
    z.tonic_kind AS tonic_kind,
    z.plain as plain,
    z.norm_title AS norm_title
FROM songs_fts5 AS s
INNER JOIN songs AS z ON z.id = s.rowid
WHERE songs_fts5 MATCH $query
ORDER BY rank`;
    return (
      await sql_promiser("exec", {
        sql: ft_query,
        bind: { $query: query },
        returnValue: "resultRows",
      })
    ).result.resultRows;
  }
  function viewer_change(evt){
    let query = new URLSearchParams(window.location.search);
    query.set("m",evt.detail.url_param);
    history.replaceState(
      null,
      null,
      window.location.pathname + "?" + query.toString()
    );
  }
  function viewer_params(){
    let query = new URLSearchParams(window.location.search);
    return query.get("m");
  }
  async function open_song(evt) {
    let src = await get_song_source_by_name(evt.detail.name);
    let result = shb_to_html(src);
    viewer_component.update_content(result.content.slice(),viewer_params());
    viewer_id = evt.detail.name;
    viewer_kind = "shb";
    if (!evt.detail.silent) {
      try {
        history.pushState(
          null,
          null,
          window.location.pathname + "?view_s=" + encodeURIComponent(evt.detail.name),
        );
      } catch (e) {}
    }
    viewer_open = true;
    result.free();
    recently_viewed_songs = push_queue_unique(evt.detail.name,recently_viewed_songs);
    return result;
  }
  async function edit_list(evt) {
    let src = await get_list_source_by_name(evt.detail.name);
    editor_component.create_document(evt.detail.name, src, "lst", false);
    if (!evt.detail.silent) {
      try {
        history.pushState(
          null,
          null,
          window.location.pathname + "?edit_l=" + encodeURIComponent(evt.detail.name),
        );
      } catch (e) {}
    }
    tabs_component.change_tab_silent("editor");
    viewer_open = false;
  }
  async function edit_song(evt) {
    let src = await get_song_source_by_name(evt.detail.name);
    editor_component.create_document(evt.detail.name, src, "shb", false);
    if (!evt.detail.silent) {
      try {
        history.pushState(
          null,
          null,
          "?edit_s=" + encodeURIComponent(evt.detail.name),
        );
      } catch (e) {}
    }
    tabs_component.change_tab_silent("editor");
    viewer_open = false;
  }
  async function open_list(evt) {
    let src = await get_list_source_by_name(evt.detail.name);
    let result = await lst_to_html(src);
    viewer_component.update_content(result.content.slice(),viewer_params());
    viewer_id = evt.detail.name;
    viewer_kind = "lst";
    requestAnimationFrame(() => {
      if (!evt.detail.silent) {
        try {
          history.pushState(
            null,
            null,
            "?view_l=" + encodeURIComponent(evt.detail.name),
          );
        } catch (e) {}
      }
      viewer_open = true;
    });
    recently_viewed_lists = push_queue_unique(evt.detail.name,recently_viewed_lists);
    //viewer_open = true;
    result.free();
    return result;
  }
  async function route(url_search_string) {
    let params = new URLSearchParams(url_search_string);
    let index = params.get("index");
    if (index) {
      switch (index) {
        case "lists":
        case "songs":
        case "editor":
        case "search":
          console.log(url_search_string);
          tabs_component.change_tab_silent(index);
          viewer_open = false;
          return;
        default:
          console.warn("Invalid route ", index);
          break;
      }
    }
    let view_s = params.get("view_s");
    if (view_s) {
      tabs_component.change_tab_silent("songs");
      open_song({ detail: { name: view_s, silent: true } }).catch((e) => {
        console.warn("Can't display song ", view_s, e);
      });
      viewer_open = true;
      return;
    }
    let view_l = params.get("view_l");
    if (view_l) {
      tabs_component.change_tab_silent("lists");
      open_list({ detail: { name: view_l, silent: true } }).catch((e) => {
        console.warn("Can't display list ", view_l, e);
      });
      viewer_open = true;
      return;
    }
    if (params.has("edit_s") || params.has("edit_l")) {
      tabs_component.change_tab_silent("editor");
      viewer_open = false;
      return;
    }
    if (params.has("connect")) {
      await viewer_component.try_connection(params.get("connect"));
    }
    //Default
    tabs_component.change_tab_silent("lists");
    viewer_open = false;
  }

  function change_tab(evt) {
    if (new URLSearchParams(window.location.search).has("index")) {
      history.replaceState(null, null, "?index=" + evt.detail.name);
    } else {
      try {
        history.pushState(null, null, "?index=" + evt.detail.name);
      } catch (e) {}
    }
    viewer_open = false;
  }
  //Why codemirror, why
  function find_app_style() {
    let regex = /^\s*:root/;
    let elems = document.head.getElementsByTagName("style");
    for (let i = 0; i < elems.length; i++) {
      if (regex.test(elems[i].innerHTML)) {
        return elems[i].innerHTML;
      }
    }
    throw new Error("Can't find app style element");
  }
  function compress_text(txt) {
    let compressed = window.libbsc.bsc_compress_u8(window.fflate.strToU8(txt));
    return window.fflate.encode_base85(compressed);
  }
  //I can't believe this works
  async function create_html_download_string(include_db = true) {
    let serialized_db_data;
    if(include_db){
      serialized_db_data = serialize_db(new Uint8Array(await get_db_array_buffer()))
    }
    /*
    if (!serialized_db_data) {
      await write_db_to_html();
      serialized_db_data = document.getElementById("serialized_db");
    } else if (true || db_modified) {
      //TODO : FIX db_modified false negative
      await write_db_to_html();
      await store_db_to_localstorage(sql_promiser);
    }*/

    let app_result = compress_text(
      document.head.getElementsByTagName("script")[0].innerHTML,
    );
    let style_result = compress_text(find_app_style());

    let script =
      "<" +
      "script id='boot'>" +
      bsc_helper_str +
      "\n" +
      boot_compressor_str +
      "</" +
      "script>";

    const string = `
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
</head>
<body>
<div id="boot-msg"></div>
${script}
<div id="__enc__" style="display:none">${app_result}</div>
<div id="__sty__" style="display:none">${style_result}</div>
<div id=\"app\"></div>
${serialized_db_data ? '<span id="serialized_db" style="display:none">' + serialized_db_data + '</span>': ""}
${document.getElementById("fflate").outerHTML}
${document.getElementById("bsc-js").outerHTML}
${document.getElementById("bsc-wasm").outerHTML}
</body>
</html>`;
    return string;
  }
  async function pack_download_app() {
    await editor_component.save_unsaved();
    let string = await create_html_download_string();
    let file = new Blob([string], { type: "text/html" });
    let link = document.createElement("a");
    link.download = "shb-" + get_date_string() + ".html";
    link.href = window.URL.createObjectURL(file);
    link.style.display = "none";

    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    db_modified = false;
  }
  async function zip_download_app() {
    await editor_component.save_unsaved();
    let string = await create_html_download_string(false);
    let db = await get_db_array_buffer();
    let zip = {
      "index.html": window.fflate.strToU8(string),
      "_worker.js" : window.fflate.strToU8(cf_worker_str),
      "db.sqlite" : db
    };
    let download = window.fflate.zipSync(zip);
    let file = new Blob([download], { type: "application/zip" });
    let link = document.createElement("a");
    link.download = "shb-" + get_date_string() + ".zip";
    link.href = window.URL.createObjectURL(file);
    link.style.display = "none";
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    db_modified = false;
  }
  async function upload_request(){
    await editor_component.save_unsaved();
    let sqlite_response = (
      await sql_promiser({
        type: "export",
      })
    ).result;
    let body = window.libbsc.bsc_compress_u8(new Uint8Array(sqlite_response.byteArray))
    let bin_str = Array.from(new TextEncoder().encode("default:" + cf_secret), (byte) =>
      String.fromCodePoint(byte),
    ).join("");
    await fetch(window.location.origin+"/kvapi/put",{
      method : "PUT",
      body : body,
      headers : {
        'Authorization':'Basic '+btoa(bin_str)
      }
    })
    .then((resp)=>{
      if (resp.status === 200){
        tab_upload_open = false;
      }else{
        password_error = true;
      }
    })
    .catch(()=>{password_error = true;});

  }
  async function on_search_input(evt) {
    let q = evt.target.value;
    q = q.normalize("NFD").replace(/[^:+()\s\p{L}\p{N}]/gu, "");
    if (q == "") {
      search_results = [];
    } else {
      //TODO
      if (/[a-zA-Z0-9]/.test(q.slice(-1))) {
        q = q + "*";
      }
      search_results = await ft_search(q);
    }
  }
  function get_date_string() {
    let date = new Date();
    return date.toISOString().slice(0, 10);
  }
  function next_song_page() {
    if ((current_song_page + 1) * items_per_page < total_songs) {
      current_song_page++;
      update_index();
    }
  }
  function prev_song_page() {
    if (current_song_page > 0) {
      current_song_page--;
      update_index();
    }
  }
  function next_list_page() {
    console.log({ current_list_page, items_per_page, total_lists });
    if ((current_list_page + 1) * items_per_page < total_lists) {
      current_list_page++;
      update_index();
    }
  }
  function prev_list_page() {
    if (current_list_page > 0) {
      current_list_page--;
      update_index();
    }
  }
  function push_queue_unique(elem,queue,max=max_recent){
    let elem_pos = queue.indexOf(elem);
    if(elem_pos > -1){
      //Argh splice modifies and taking by 'reference' is cumbersome
      queue = queue.slice();
      queue.splice(elem_pos,1);
      return [elem, ...queue];
    }
    if(queue.length >= max){
      return [elem, ...queue.slice(0,max-1)];
    }
    return [elem,...queue];
  }
</script>

<div bind:this={container}>
  <Tabs
    on:change_tab={change_tab}
    on:upload_app={()=>{
      tab_upload_open = !tab_upload_open;
      password_error = false;
    }}
    on:download_app={() => {
      tab_save_open = !tab_save_open;
    }}
    {tab_save_open}
    {tab_upload_open}
    bind:this={tabs_component}
  >
    <div class="tab-upload" slot="upload" transition:fade>
      {#if password_error}❌{/if}
      <input id="upload-pass" type="password" bind:value={cf_secret} placeholder="contraseña"/>
      <button on:click={upload_request}>Subir</button>
    </div>
    <div class="tab-save" slot="download" transition:fade>
      <button
        on:click={() => {
          pack_download_app();
          tab_save_open = false;
        }}>HTML aplicación</button
      >
      <button
        on:click={() => {
          download_db();
          tab_save_open = false;
        }}>SQLite base de datos</button
      >
      <button
        on:click={() => {
          zip_download_app();
          tab_save_open = false;
        }}>ZIP aplicación</button
      >
    </div>
    <div slot="lists">
      <div class="recent_bar">
        {#if recently_viewed_lists}
          <span>Ultimos vistos:</span>
          {#each recently_viewed_lists as r_list }
            <button on:click={()=>{open_list( { detail: {name: r_list} })}}>{r_list}</button>
          {/each}
        {/if}
      </div>
      <ListIndex
        on:next_page={next_list_page}
        on:prev_page={prev_list_page}
        on:open_list={open_list}
        on:edit_list={edit_list}
        total_pages={Math.ceil(total_lists / items_per_page)}
        current_page={current_list_page + 1}
        lists={list_index}
        hidden={viewer_open}
      ></ListIndex>
    </div>

    <div slot="songs">
      <div class="recent_bar">
        {#if recently_viewed_songs}
        <span>Ultimos vistos:</span>
          {#each recently_viewed_songs as r_song }
            <button on:click={()=>{open_list({ detail: {name: r_song} })}}>{r_song}</button>
          {/each}
        {/if}
      </div>
      <SongIndex
        on:next_page={next_song_page}
        on:prev_page={prev_song_page}
        on:open_song={open_song}
        on:edit_song={edit_song}
        total_pages={Math.ceil(total_songs / items_per_page)}
        current_page={current_song_page + 1}
        songs={song_index}
        slot="songs"
        hidden={viewer_open}
      ></SongIndex>
    </div>

    <Search
      slot="search"
      on:input={on_search_input}
      on:open_song={open_song}
      on:edit_song={edit_song}
      results={search_results}
      hidden={viewer_open}
    ></Search>
    <Editor
      hidden={viewer_open}
      slot="editor"
      bind:this={editor_component}
      on:update={() => {
        update_index();
        db_modified = true;
      }}
    ></Editor>
  </Tabs>
  {#if progress < 100}
    <Progress {progress}></Progress>
  {/if}
  <ViewWrapper
    bind:this={viewer_component}
    on:edit_song={edit_song}
    on:edit_list={edit_list}
    on:shb_modified={viewer_change}
    hidden={!viewer_open}
    id={viewer_id}
    kind={viewer_kind}
  ></ViewWrapper>
  <div class="danger-zone">
    <button on:click={download_db}>Descargar DB</button>
    <span class="db-area">
      <label for="db_upload">⚠️ Cargar DB</label>
      <input type="file" id="db_upload" on:change={upload_db} />
      {#if database_reloaded_state === "OK"}✅
      {:else if database_reloaded_state === "ERROR"}❌{/if}
    </span>
    <span class="db-merge-area">
      <label for="db_merge">⚠️ Mezclar DB</label>
      <input type="file" id="db_merge" on:change={merge_db} />
    </span>
    <button
      on:click={() => {
        window.localStorage.removeItem("db");
        location.reload();
      }}>⚠️ Borrar datos temporales</button
    >
    <span id="tagname">Shibim 5.0 <i>Axapusco</i></span>
  </div>
</div>
<svelte:document
  on:visibilitychange={async () => {
    if (document.visibilityState === "hidden" && db_modified) {
      await store_db_to_localstorage(sql_promiser);
      db_modified = false;
    }
  }}
/>
<svelte:window
  on:beforeunload={(evt) => {
    if (db_modified) {
      store_db_to_localstorage(sql_promiser).then(() => {
        db_modified = false;
      });
      evt.preventDefault();
      return true;
    }
    return false;
  }}
/>

<style>
  .db-area,
  .db-merge-area {
    background-color: var(--bgcolor);
    border-radius: 10px;
    padding: 5px;
    border: 1px solid #8b8b8b8a;
    display: inline-block;
  }
  .danger-zone {
    margin-top: 40px;
  }
  #tagname {
    font-size: 0.5em;
    display: inline-block;
  }
  .tab-save {
    text-align: center;
  }
  .tab-upload {
    text-align: center;
  }
  .recent_bar{
    border-bottom: 1px solid var(--bar-color);
  }
</style>
