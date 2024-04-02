<script lang="ts">
    export let hidden = false;
    import { SHBStreamTokenizer, LSTStreamTokenizer } from "./lib/parser";
    import {
        EditorView,
        keymap,
        highlightSpecialChars,
        highlightActiveLine,
        dropCursor,
        ViewUpdate,
        lineNumbers,
        highlightActiveLineGutter,
    } from "@codemirror/view";
    import { EditorState, Compartment } from "@codemirror/state";
    import {
        syntaxHighlighting,
        indentOnInput,
        bracketMatching,
        foldGutter,
        StreamLanguage
    } from "@codemirror/language";
    import {setDiagnostics, type Diagnostic, lintGutter} from "@codemirror/lint"
    import {
        defaultKeymap,
        history,
        historyKeymap,
    } from "@codemirror/commands";
    import {
        searchKeymap,
        highlightSelectionMatches,
    } from "@codemirror/search";
    import {
        autocompletion,
        CompletionContext,
        type CompletionResult
    } from "@codemirror/autocomplete";
    import { tags } from "@lezer/highlight";
    import { HighlightStyle } from "@codemirror/language";
    import { onMount, getContext, tick} from "svelte";
    import {
        shb_to_db_info,
        shb_to_html,
        lst_to_html,
        lst_to_array,
    } from "shibim-js/shibim_js";
    import { update_abc } from "./lib/abc";
    import { createEventDispatcher } from 'svelte';
	const dispatch = createEventDispatcher();
    import { fade } from "svelte/transition";
    type DocData = {
            view: EditorView;
            dom: Element;
            lang: string;
            preview: Element;
            name: string;
            is_new: boolean;
            is_modified: boolean;
        };
    type DocMap = {
        [key in number]: DocData
    };
    const lang_compartment = new Compartment();
    const lang_shb = StreamLanguage.define(new SHBStreamTokenizer());
    const lang_lst = StreamLanguage.define(new LSTStreamTokenizer());
    let container: Element;
    let db_exec: (b: any) => any = getContext("db");
    let tabs: number[] = [];
    let doc_i = 0;
    let view_stack: number[] = [];
    let documents: DocMap = {};
    let name_map: Record <string,number> = {};
    let save_tool_action = "";
    let save_name = "";
    let ovw_delete_target : {name:string, lang:string};
    let highlighter = HighlightStyle.define([
        { tag: tags.meta, color: "var(--deemph-color)" },
        { tag: tags.link, textDecoration: "underline" },
        { tag: tags.heading, textDecoration: "underline", fontWeight: "bold" },
        { tag: tags.emphasis, fontStyle: "italic" },
        { tag: tags.strong, fontWeight: "bold" },
        { tag: tags.strikethrough, textDecoration: "line-through" },
        { tag: tags.keyword, color: "var(--emphasis-color)" },
        { tag: tags.punctuation, color: "#888" },
        {
            tag: [
                tags.atom,
                tags.bool,
                tags.url,
                tags.contentSeparator,
                tags.labelName,
            ],
            color: "#219",
        },
        {
            tag: [tags.literal, tags.inserted],
            color: "var(--chord-edit-color)",
            fontWeight: "bold",
        },
        {
            tag: [tags.string, tags.deleted],
            color: "var(--max-emphasis-color)",
            fontWeight: "bold",
        },
        {
            tag: [tags.regexp, tags.escape, tags.special(tags.string)],
            color: "#e40",
        },
        { tag: tags.definition(tags.variableName), color: "#00f" },
        { tag: tags.local(tags.variableName), color: "#30a" },
        { tag: [tags.typeName, tags.namespace], color: "#085" },
        { tag: tags.className, color: "#167" },
        {
            tag: [tags.special(tags.variableName), tags.macroName],
            color: "#256",
        },
        { tag: tags.definition(tags.propertyName), color: "#00c" },
        { tag: tags.comment, color: "#940" },
        { tag: tags.invalid, color: "#f00" },
    ]);

    async function get_song_suggestions(query: string) {
        query = query.replace("ñ","n");
        if (/[a-zA-Z0-9]/.test(query.slice(-1))) {
            query = query + "*";
        }
        if(/[^a-zA-Z0-9+*\s]/.test(query)){
            console.warn("song completition fed wrong kind of query");
            return [];
        }
        const ft_query = `SELECT s.rowid AS id,
    s.name AS handle,
    z.title AS title,
    s.subtitle AS subtitle,
    z.tonic AS tonic,
    z.tonic_kind AS tonic_kind,
    z.plain as plain,
    z.sections as sections
FROM songs_fts5 AS s
INNER JOIN songs AS z ON z.id = s.rowid
WHERE songs_fts5 MATCH $query
ORDER BY rank`;
        return await db_exec({
            sql: ft_query,
            bind: { $query: query },
            rowMode: "object",
            returnValue: "resultRows",
        }).catch(()=>[]);
    }
    function getDiagnostics() : Diagnostic[]{
        return []
    }
    async function lst_autocompletion(ctx: CompletionContext) : Promise<CompletionResult | null> {
        if (
            ctx.tokenBefore(["link"]) ||
            (ctx.matchBefore(/^[^|<>]*/) && ctx.explicit)
        ) {
            let word = ctx.matchBefore(/.*/);
            if (!word || (word.from == word.to && !ctx.explicit)) {
                return null;
            }
            let opts = (await get_song_suggestions(word.text)).map(
                (song: any) => {
                    return {
                        label: song.handle,
                        displayLabel: song.title,
                        info : ()=>{
                            let div = document.createElement("pre");
                            div.className = "cm-editor-suggestion-info-box";
                            div.textContent = song.sections;
                            return div;
                        }
                    };
                },
            );
            //console.log({q:word.text,r:opts});
            return {
                from: word.from,
                options: opts,
                filter: false,
            };
        } else {
            if (ctx.matchBefore(/order( *):[ ~\w]+/)) {
                let word = ctx.matchBefore(/(\w|~)*/);
                if(word === null){
                    return null;
                }
                let i_line = ctx.state.selection.main.head;
                let song_name;
                while (i_line > 0) {
                    let line = ctx.state.doc.lineAt(i_line).text;
                    let song_match = line.match(/^[^:|]+/);
                    if (song_match) {
                        song_name = song_match[0].trim();
                        if (song_name) {
                            break;
                        }
                    }
                    i_line--;
                }
                if (!song_name) {
                    return null;
                }
                let song_data : Array<Array<string>> = await db_exec({
                    sql : "select sections from songs where name = ?",
                    bind : [song_name],
                    returnValue : "resultRows"
                });
                let sections = song_data[0][0].split("\n").map(v=>v.split("|")).filter(v=>!!v[0] || !!v[1]);
                return {
                    from : word.from,
                    options : sections.map(sect=>{return{
                        label : sect[0],
                        detail : sect[1]
                    }})
                };
            } else if (ctx.matchBefore(/\|\s*\w*/)) {
                let word = ctx.matchBefore(/\w*/);
                if (!word) {
                    return null;
                }
                return {
                    from: word.from,
                    options: [
                        { label: "tonic :", detail: " tono" },
                        { label: "order :", detail: " orden de secciones" },
                        { label: "join :", detail: " unir con anterior" },
                    ],
                };
            }
        }
        return null;
    }
    export function focus(){
        if(view_stack.length > 0){
            documents[view_stack[view_stack.length-1]].view.focus();
        }
    }
    function create_view(
        content = "",
        lang = "lst",
        on_modify?: (a: ViewUpdate) => void,
    ) {
        let language;
        let complet;
        switch (lang) {
            case "shb":
                language = lang_compartment.of(lang_shb);
                complet = autocompletion({
                    activateOnTyping: false,
                });
                break;
            case "lst":
                language = lang_compartment.of(lang_lst);
                complet = autocompletion({
                    activateOnTyping: true,
                    override: [lst_autocompletion],
                });
                break;
            default:
                language = null;
                break;
        }
        let extensions: any[] = [
            lineNumbers(),
            highlightActiveLineGutter(),
            history(),
            dropCursor(),
            lintGutter(),
            //EditorState.allowMultipleSelections.of(true),
            indentOnInput(),
            syntaxHighlighting(highlighter, { fallback: true }),
            complet,
            //rectangularSelection(),
            highlightActiveLine(),
            highlightSelectionMatches(),
            keymap.of([
                ...defaultKeymap,
                ...searchKeymap,
                ...historyKeymap,
                {
                    key: "Ctrl-.",
                    run: (view) => {
                        view.dispatch(view.state.replaceSelection("·"));
                        return true;
                    },
                },
                {
                    key: "Ctrl-,",
                    run: (view) => {
                        view.dispatch(view.state.replaceSelection("`"));
                        return true;
                    },
                },
                //...foldKeymap,
                //...completionKeymap,
                //...lintKeymap
            ]),
            EditorView.inputHandler.of((view,from,to,text,insert)=>{
                if (from !== to){
                    return false;
                }
                let prev_char = view.state.sliceDoc(from-1,from);
                if (text === ","){
                    if (prev_char === ","){
                        view.dispatch({
                            changes: [{from:from-1, to, insert: '`'}],
                        });
                        return true;
                    } else if (prev_char === "`"){
                        view.dispatch({
                            changes: [{from:from-1, to, insert: '|'}],
                        });
                        return true;
                    }
                }
                if (text === "."){
                    if (prev_char === "."){
                        view.dispatch({
                            changes: [{from:from-1, to, insert: '·'}],
                        });
                        return true;
                    } else if (prev_char === "·"){
                        view.dispatch({
                            changes: [{from:from-1, to, insert: '...'}],
                        });
                        return true;
                    }
                }
                return false;
            }),
            language,
        ];
        if (on_modify) {
            extensions.push(EditorView.updateListener.of(on_modify));
        }
        let state = EditorState.create({
            doc: content,
            extensions: extensions,
        });
        return new EditorView({
            state: state,
        });
    }
    export function create_document(
        name: string,
        content: string,
        lang: string,
        is_new: boolean
    ) {
        add_document(name, content, lang, is_new);
    }
    function new_document() {
        create_document("", "", "lst",true);
        requestAnimationFrame(()=>focus());
    }
    
    function update_preview(doc : DocData){
        if (doc.lang === "shb") {
            let res = shb_to_html(doc.view.state.doc.toString());
            doc.preview.innerHTML = res.content;
            let diag : Diagnostic[] = res.errors.map((e : any)=>{
                return {
                    from : e.from,
                    to : e.to,
                    severity : "error",
                    message : e.message
                };
            });
            doc.view.dispatch(setDiagnostics(doc.view.state,diag));
            console.log(res.errors);
            tick().then(()=>{
                update_abc(container, true);
            });
        } else if (doc.lang === "lst"){
            lst_to_html(doc.view.state.doc.toString()).then((res) => {
                doc.preview.innerHTML = res.content;
                let diag : Diagnostic[] = res.s_errors.concat(res.l_errors).map((e : any)=>{
                    return {
                        from : e.from,
                        to : e.to,
                        severity : "error",
                        message : e.message
                    };
                });
                doc.view.dispatch(setDiagnostics(doc.view.state,diag));
                res.free();
            });
        }
    }
    function add_document(name: string, content: string, lang: string, is_new : boolean): number {
        let prev_id = name_map[name];
        if (prev_id !== undefined && !is_new){
            switch_document(prev_id);
            return prev_id;
        }
        let new_doc : DocData = {
            name: name,
            dom: document.createElement("div"),
            lang: lang,
            preview: document.createElement("div"),
            // @ts-ignore
            view: undefined,
            is_new: is_new,
            is_modified: false,
        };
        let timeout : number;
        let doc = create_view(content, lang,(function(this : DocData, update : ViewUpdate){
            if(update.docChanged){
                if (!this.is_modified){
                    this.is_modified = true;
                    tabs = tabs;
                }
                if (timeout) {
                    clearTimeout(timeout);
                }
                timeout = setTimeout(() => {
                    update_preview(this)
                }, 1000);
            }
        }).bind(new_doc));
        new_doc.view = doc;
        new_doc.dom.className = "editor-tab-container";
        new_doc.dom.appendChild(doc.dom);
        new_doc.dom.appendChild(new_doc.preview);
        if (lang === "shb") {
            let res = shb_to_html(doc.state.doc.toString());
            new_doc.preview.className = "u-song-container";
            new_doc.preview.innerHTML = res.content;
            tick().then(()=>{
                update_abc(container, true);
            });
        } else if (lang === "lst"){
            lst_to_html(doc.state.doc.toString()).then((res) => {
                new_doc.preview.className = "u-song-container";
                new_doc.preview.innerHTML = res.content;
                res.free();
            });
        }
        documents[doc_i] = new_doc;
        tabs = [...tabs, doc_i];
        name_map[name] = doc_i;
        if (view_stack.length > 0) {
            documents[view_stack[view_stack.length - 1]].dom.remove();
        }
        view_stack.push(doc_i);
        container.appendChild(new_doc.dom);
        doc_i = doc_i + 1;
        return doc_i - 1;
    }
    function switch_document(id: number | string) {
        save_tool_action = "";
        if (typeof id === "string") {
            id = parseInt(id);
        }
        if (Number.isInteger(id)) {
            documents[view_stack[view_stack.length - 1]].dom.remove();
            let idx = view_stack.indexOf(id);
            let view = view_stack.splice(idx, 1)[0];
            view_stack = [...view_stack, view];
            container.appendChild(documents[view].dom);
        }
        tabs = tabs;
    }
    async function save_content(
        name: string,
        kind: string,
        content: string,
        force: boolean,
    ): Promise<boolean> {
        console.debug("saving ", {name,kind,force});
        if (kind === "shb") {
            if (force){
                let processed = shb_to_db_info(content);
                await db_exec({
                    sql: `
INSERT INTO songs (name, title, norm_title, source, plain, subtitle, sections, tonic, tonic_kind)
VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
ON CONFLICT (name) DO UPDATE SET title = ?2, norm_title = ?3, source = ?4, plain = ?5, subtitle = ?6, sections = ?7, tonic = ?8, tonic_kind = ?9;`,
                    bind: [
                        name,
                        processed.title,
                        processed.norm_title,
                        content,
                        processed.plain,
                        processed.subtitle,
                        processed.sections,
                        processed.tonic,
                        processed.tonic_kind,
                    ],
                });
                processed.free();
            }else{
                let processed = shb_to_db_info(content);
                let rc = await db_exec({
                    sql: `
INSERT INTO songs (name, title, norm_title, source, plain, subtitle, sections, tonic, tonic_kind)
VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9);`,
                    bind: [
                        name,
                        processed.title,
                        processed.norm_title,
                        content,
                        processed.plain,
                        processed.subtitle,
                        processed.sections,
                        processed.tonic,
                        processed.tonic_kind,
                    ],
                }).then(()=>true).catch((err: any) => {
                    if (
                        err.result.message.startsWith(
                            "SQLITE_CONSTRAINT_UNIQUE",
                        )
                    ) {
                        console.debug("File already exists: ", name);
                        return false;
                    } else {
                        throw err;
                    }
                });
                if (rc === false) {
                    return false;
                }
            }
        } else if (kind === "lst") {
            let processed: string[] = await lst_to_array(content);
            let id;
            if (!force) {
                const base_query =
                    "INSERT INTO lists (name, source) VALUES (?1,?2) RETURNING id;";
                id = await db_exec({
                    sql: base_query,
                    bind: [name, content],
                    returnValue: "resultRows",
                }).catch((err: any) => {
                    if (
                        err.result.message.startsWith(
                            "SQLITE_CONSTRAINT_UNIQUE",
                        )
                    ) {
                        console.debug("File already exists: ", name);
                        return false;
                    } else {
                        throw err;
                    }
                });
                if (id === false) {
                    return false;
                }
            }else{
                const base_query =
                    "INSERT INTO lists (name, source) VALUES (?1,?2) ON CONFLICT (name) DO UPDATE SET source = ?2 RETURNING id;";
                id = await db_exec({
                    sql: base_query,
                    bind: [name, content],
                    returnValue: "resultRows",
                });
            }
            id = id[0][0];
            await db_exec({
                sql: "DELETE FROM lists_songs WHERE list = ?",
                bind: [id],
            });
            const id_query = `WITH instr AS (\n SELECT ? AS name, 0 as pos`;
            if (processed.length < 1) {
            } else {
                let query = id_query;
                for (let i = 1; i < processed.length; i++) {
                    query += `\nUNION ALL SELECT ?, ${i}`;
                }
                query += `)
INSERT INTO lists_songs (song, list, position)
SELECT s.id AS song, ${id} AS list, i.pos AS position FROM instr i
JOIN songs s WHERE i.name = s.name
ON CONFLICT (list, song) DO NOTHING`;
                await db_exec({
                    sql: query,
                    bind: processed,
                    returnValue: "resultRows",
                });
            }
        }
        dispatch("update",{name, kind,action:"write"});
        return true;
    }
    async function save_current(_evt: any) {
        let last = view_stack[view_stack.length - 1];
        if (last !== undefined) {
            let doc = documents[last];
            if (!doc.is_new) {
                let ok = await save_content(
                    doc.name,
                    doc.lang,
                    doc.view.state.doc.toString(),
                    !doc.is_new,
                );
                if (!ok) {
                    save_name = doc.name;
                    save_tool_action = "OVERWRITE_RENAME"
                }else{
                    doc.is_modified = false;
                    tabs = tabs;
                }
            } else {
                save_name = "";
                save_tool_action = "NEW";
            }
        }
    }
    async function rename_start(){
        let last = view_stack[view_stack.length - 1];
        if (last === undefined) {
            return;
        }
        let doc = documents[last];
        save_name = doc.name;
        if (doc.is_new){
            save_tool_action = "NEW";
        }else{
            save_tool_action = "RENAME";
        }
    }
    async function save_new(kind : string) {
        let last = view_stack[view_stack.length - 1];
        if (last === undefined) {
            return;
        }
        let doc = documents[last];
        if (kind === "shb" ){
            doc.view.dispatch({
                effects : lang_compartment.reconfigure(lang_shb)
            });
        }else if (kind === "lst"){
            doc.view.dispatch({
                effects : lang_compartment.reconfigure(lang_lst)
            });
        }else{
            return;
        }
        doc.lang = kind;
        update_preview(doc);
        let ok = await save_content(
                save_name,
                kind,
                doc.view.state.doc.toString(),
                false,
        );
        //TODO: ERROR
        if (ok){
            save_tool_action = "";
            doc.name = save_name;
            doc.is_modified = false;
            doc.is_new = false;
            tabs = tabs;
        }else{
            ovw_delete_target = {name:save_name, lang:kind}
            save_tool_action = "OVERWRITE_RENAME"
        }
    }
    async function delete_content(name : string, kind:string){
        if (kind === "shb"){
            await db_exec({
                sql : "DELETE FROM songs WHERE name = ?",
                bind : [name]
            })
        } else if(kind === "lst"){
            await db_exec({
                sql : "DELETE FROM lists WHERE name = ?",
                bind : [name]
            })
        }
        dispatch("update",{name, kind,action:"delete"});
    }
    async function save_rename(kind : string) {
        let last = view_stack[view_stack.length - 1];
        if (last === undefined) {
            return;
        }
        let doc = documents[last];
        if (kind === "shb" ){
            doc.view.dispatch({
                effects : lang_compartment.reconfigure(lang_shb)
            });
        }else if (kind === "lst"){
            doc.view.dispatch({
                effects : lang_compartment.reconfigure(lang_lst)
            });
        }else{
            return;
        }
        let ok = await save_content(
                save_name,
                kind,
                doc.view.state.doc.toString(),
                false,
        );
        //TODO: ERROR
        if (ok){
            if (!doc.is_new){
                await delete_content(doc.name, doc.lang);
            }
            doc.lang = kind;
            save_tool_action = "";
            doc.name = save_name;
            doc.is_modified = false;
            doc.is_new = false;
            update_preview(doc);
            tabs = tabs;
        }else{
            ovw_delete_target = {name:save_name, lang:kind};
            save_tool_action = "OVERWRITE_ONLY"
        }
    }
    async function save_old() {
        let last = view_stack[view_stack.length - 1];
        if (last === undefined) {
            return;
        }
        let doc = documents[last];
        await save_content(
                save_name,
                doc.lang,
                doc.view.state.doc.toString(),
                true,
        );
        doc.is_modified = false;
        tabs = tabs;
    }
    async function confirm_overwrite(){
        let last = view_stack[view_stack.length - 1];
        if (last === undefined) {
            return;
        }
        let doc = documents[last];
        await save_content(
            ovw_delete_target.name,
            ovw_delete_target.lang,
            doc.view.state.doc.toString(),
            true,
        );
        doc.name = ovw_delete_target.name;
        doc.lang = ovw_delete_target.lang;
        doc.is_modified = false;
        doc.is_new = false;
        save_tool_action = "";
        tabs = tabs;
    }
    async function confirm_overwrite_rename(){
        let last = view_stack[view_stack.length - 1];
        if (last === undefined) {
            return;
        }
        let doc = documents[last];
        await save_content(
            save_name,
            ovw_delete_target.lang,
            doc.view.state.doc.toString(),
            true,
        );
        doc.name = save_name;
        doc.lang = ovw_delete_target.lang;
        doc.is_modified = false;
        doc.is_new = false;
        save_tool_action = "";
        update_preview(doc);
        tabs = tabs;
    }
    async function confirm_delete(){
        let last = view_stack[view_stack.length - 1];
        if (last === undefined) {
            return;
        }
        let doc = documents[last];
        if(!doc.is_new){
            await delete_content(doc.name,doc.lang);
        }
        close_tab(last,true);
        save_tool_action = "";
    }
    async function confirm_overwrite_delete(){
        let last = view_stack[view_stack.length - 1];
        if (last === undefined) {
            return;
        }
        let doc = documents[last];
        await save_content(
                    ovw_delete_target.name,
                    ovw_delete_target.lang,
                    doc.view.state.doc.toString(),
                    true,
        );
        await delete_content(
            doc.name,
            doc.lang
        );
        doc.name = ovw_delete_target.name;
        doc.lang = ovw_delete_target.lang;
        save_tool_action = "";
        tabs = tabs;
    }
    async function confirm_close_current() {
        let last = view_stack[view_stack.length - 1];
        if (last === undefined) {
            return;
        }
        await close_tab(last,true);
        save_tool_action = "";
    }
    async function close_tab(tab: number, force: boolean = false) {
        console.debug("closing tab ",{tab,force})
        if (documents[tab].is_modified && !force) {
            switch_document(tab);
            save_tool_action = "CLOSE";
            //TODO
        } else {
            let v_i = view_stack.indexOf(tab);
            let t_i = tabs.indexOf(tab);
            tabs.splice(t_i, 1);
            if (v_i == view_stack.length - 1) {
                if (tabs.length > 0) {
                    switch_document(view_stack[view_stack.length - 2]);
                    view_stack.splice(view_stack.length - 2, 1);
                    delete name_map[documents[tab].name];
                    tabs = tabs;
                    return;
                } else {
                    documents[tab].dom.remove();
                }
            }
            view_stack.splice(v_i, 1);
            delete name_map[documents[tab].name];
            tabs = tabs;
        }
    }
    function close_save_tool() {
        save_tool_action = "";
    }
    onMount(() => {});
    function has_unsaved(){
        for(let tab in documents){
            if(documents[tab].is_modified){
                return true;
            }
        }
        return false;
    }
    export async function save_unsaved(){
        for(let tab in documents){
            let doc = documents[tab];
            if(doc.is_modified && !doc.is_new){
                await save_content(
                    doc.name,
                    doc.lang,
                    doc.view.state.doc.toString(),
                    !doc.is_new,
                )
                doc.is_modified = false;
                tabs = tabs;
            }
        }
    }
    function beforeunload(evt : Event){
        if (has_unsaved()){
            evt.preventDefault();
            return true;
        }
        return false;
    }
</script>

<div class="toolbar-parent" class:hidden={hidden}>
    <div class="toolbar">
        <button on:click={new_document}>🗒️ Nuevo</button>
        <button on:click={save_current}>💾 Guardar</button>
        <button on:click={()=>save_tool_action = "NEW"}>📋 Guardar como</button>
        <button on:click={rename_start}>🔤 Renombrar</button>
        <button on:click={()=>save_tool_action="DELETE"}>🗑️ Eliminar</button>
    </div>
    {#if save_tool_action}
        <div class="save-tool" transition:fade>
            {#if save_tool_action == "NEW"}
                <label for="tool-new-name">Nombre: </label>
                <!-- svelte-ignore a11y-autofocus -->
                <input type="text" bind:value={save_name} id="tool-new-name" name="name" autofocus/>
                <div class="save-button-container">
                    <button on:click={()=>save_new("shb")}>Guardar como canción</button>
                    <button on:click={()=>save_new("lst")}>Guardar como lista</button>
                    <button on:click={close_save_tool}>Cancelar</button>
                </div>
            {:else if save_tool_action == "RENAME"}
                <label for="tool-new-name">Nuevo nombre: </label>
                <!-- svelte-ignore a11y-autofocus -->
                <input type="text" bind:value={save_name} id="tool-new-name" name="name" autofocus/>
                <div class="save-button-container">
                    <button on:click={()=>save_rename("shb")}>Guardar como canción</button>
                    <button on:click={()=>save_rename("lst")}>Guardar como lista</button>
                    <button on:click={close_save_tool}>Cancelar</button>
                </div>
            {:else if save_tool_action == "OVERWRITE_RENAME"}
                <label for="tool-owr-name">El archivo ya existe: </label>
                <input type="text" bind:value={save_name} id="tool-owr-name" name="name" />
                <div class="save-button-container">
                    <button on:click={confirm_overwrite}>Sobreescribir</button>
                    <button on:click={confirm_overwrite_rename}>Renombrar</button>
                    <button on:click={close_save_tool}>Cancelar</button>
                </div>
            {:else if save_tool_action == "OVERWRITE_ONLY"}
                <label for="tool-owr-name">El archivo ya existe: </label>
                <input type="text" bind:value={save_name} id="tool-owr-name" name="name" />
                <div class="save-button-container">
                    <button on:click={confirm_overwrite_delete}>Sobreescribir</button>
                    <button on:click={close_save_tool}>Cancelar</button>
                </div>
            {:else if save_tool_action == "DELETE"}
                <span>Eliminar archivo?</span>
                <div class="save-button-container">
                    <button on:click={confirm_delete}>Eliminar</button>
                    <button on:click={close_save_tool}>Cancelar</button>
                </div>
            {:else if save_tool_action == "CLOSE"}
                <span>El archivo fue modificado: </span>
                <div class="save-button-container">
                    <button on:click={save_current}>Guardar</button>
                    <button on:click={confirm_close_current}>Descartar</button>
                    <button on:click={close_save_tool}>Cancelar</button>
                </div>
            {/if}
        </div>
    {/if}
</div>
<div class="tabs" class:hidden={hidden}>
    {#each tabs as tab}
    <span class="tabspan" class:current={view_stack[view_stack.length - 1] === tab}>
        <button class="tab-btn" on:click={() => switch_document(tab)}  class:modified={documents[tab].is_modified}>
            {#if documents[tab].name}
                {documents[tab].name}
            {:else}
                {"Nuevo " + tab}
            {/if}
        </button>
        <button
            class="close"
            on:click={() => {
                close_tab(tab);
            }}>❌</button
        >
    </span>
    {/each}
</div>
<div bind:this={container}  class="workpanel" class:hidden={hidden}></div>
<svelte:window on:beforeunload={beforeunload}/>
<style>
    .hidden{
        display: none;
    }
    .save-button-container {
        display: flex;
        flex-direction: row;
        max-width: 36em;
        margin: 0 auto;
    }
    .save-button-container button {
        flex: 1 1 0;
    }
    .current{
        font-weight: bold;
    }
    .modified{
        font-style: italic;
    }
    .toolbar-parent {
        background-image: repeating-linear-gradient(
            -45deg,
            transparent,
            transparent 1.5px,
            rgba(175, 175, 175, 0.25) 2px,
            rgba(175, 175, 175, 0.25) 3px,
            transparent 3.5px,
            transparent 5px
        );
        border-bottom: 1px solid #777;
        padding-bottom: 2px;
        background-color: var(--bgcolor);
        text-align: center;
    }
    .save-tool {
        margin: 5px;
        border-radius: 10px;
        border: 2px solid;
        padding: 5px;
    }
    .tabs {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
    }
    .tab-btn {
        margin-right: 0;
        padding-right: 0.1em;
        border-bottom-right-radius: 0;
        border-top-right-radius: 0;
        border-right: none;
    }
    .close {
        margin-left: 0;
        padding-left: 0.1em;
        border-bottom-left-radius: 0;
        border-top-left-radius: 0;
        border-left: none;
        font-size: 0.75em;
    }
    .tabspan{
        display: inline-block;
        display: flex;
    }
    .tabspan.current{
        box-shadow: inset var(--main-font-color) 0px 0px 4px;
        border-radius: 10px;
    }
</style>
