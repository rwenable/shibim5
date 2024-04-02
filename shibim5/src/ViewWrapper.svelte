<script>
    export let content = "";
    export let id = "";
    export let kind = "";
    export let hidden = false;
    import {set_up_local, set_websocket, try_default_websocket, create_websocket, scrollNextSongView, scrollPrevSongView, set_present_channel, setPresentationClick, setSharpSetting, setPresFontSize} from "./lib/song";
    import {onMount, createEventDispatcher} from "svelte";
    import {get_presentation_html_string} from "./present";
    import {update_abc} from "./lib/abc"
    let no_margin = false;
    let container;
    let font_size = "inherit";
    let opened_presentation = false;
    let pres_font_size = 18;
    let connect_state = "CLOSED";
    let search_cancel = ()=>{};
    let ip_search_text = "..."
    let ip_text = "";
    let connect_result_text = ""
    const dispatch = createEventDispatcher();
    $: content, requestAnimationFrame(()=>{
            set_up_local(container);
            setPresentationClick(container);
            update_abc(container,false);
            let songs = container.getElementsByClassName("u-song");
            for(const song of songs){
              if (song.hasAttribute("data-id")){
                const bar = song.getElementsByClassName("util-buttons-box")[0];
                let btn = document.createElement("button");
                btn.textContent = "✏️";
                btn.className = "edit-button";
                btn.addEventListener("click",()=>{
                  dispatch("edit_song",{name:song.getAttribute("data-id")});
                  
                });
                bar.appendChild(btn);
              }
            }
        });
    onMount(() => {
      set_present_channel(container);
      try_default_websocket().then(ws=>sessionStorage.setItem("ws-server",ws.url))
      .catch(()=>console.log("Could not connect to websocket server"));
    });
    function open_presentation_page(){
      let new_w = window.open(undefined,"_blank","popup");
      new_w.document.write(get_presentation_html_string());
      opened_presentation = true;
    }
    function increment_font_size(evt){
      if (typeof font_size === 'string'){
        let current_size = getComputedStyle(container);
        font_size = parseInt(current_size.fontSize);
      }
      font_size += (2*Math.sqrt(9*font_size -8) - 7)/9;
      rescroll(evt);
    }
    function increment_pres_size(evt){
      pres_font_size += (2*Math.sqrt(9*pres_font_size -8) - 7)/9;
      setPresFontSize(pres_font_size.toString()+"px");
    }
    function decrement_pres_size(evt){
      pres_font_size -= (2/3)*Math.sqrt(pres_font_size) - 1;
      setPresFontSize(pres_font_size.toString()+"px");
    }
    function rescroll(evt){
      requestAnimationFrame(()=>{
        evt.target.scrollIntoView();
      });
    }
    function edit_top(evt){
      if (kind === "shb"){
        dispatch("edit_song",{name: id});
      }else if(kind == "lst"){
        dispatch("edit_list",{name: id});
      }
    }
    function decrement_font_size(){
      if (typeof font_size === 'string'){
        let current_size = getComputedStyle(container);
        font_size = parseInt(current_size.fontSize);
      }
      font_size -= (2/3)*Math.sqrt(font_size) - 1;
    }
    function trunc (n){
      return Math.trunc(n*10)/10;
    }
    function keypress(evt){
      let keyevent = evt.detail;
      if(hidden){
        return;
      }
      if (keyevent.code === "ArrowLeft" || keyevent.keyCode === 37){
        scrollPrevSongView();
      }
      if(keyevent.code === "ArrowRight" || keyevent.keyCode === 39){
        scrollNextSongView();
      }
    }
    export async function try_connection(new_ip = null){
      connect_result_text = ""
      if (new_ip){
        ip_text = new_ip;
      }
      let is_tls = window.location.protocol !== "http:";
      let ip_parts = ip_text.split(":");
      let protocol = is_tls ?  "wss:" : "ws:";
      if (ip_parts.length === 1){
        ip_text = ip_text + (is_tls? ":64065" : ":64064");
      }
      let uri = protocol + "//" + ip_text + "/ws";
      await create_websocket(uri,(success,ws)=>{
        if (success){
          connect_result_text = "✅"
          set_websocket(ws);
          sessionStorage.setItem("ws-server",ws.url);
        }else{
          connect_result_text="⚠️"
        }
      })
      .then((ws)=>{
        connect_result_text = "✅"
        set_websocket(ws);
        sessionStorage.setItem("ws-server",ws.url);
      })
      .catch((e)=>{
        connect_result_text = "❌";
        throw e;
      });
      
    }
    async function start_search(){
      connect_state = "SEARCH";
      return await webScanAll(
        undefined,
        {
          rtc : false,
          noRedirect : false,
          networkCallback: async function(ip,cancel) {
            search_cancel = cancel;
            console.log(`Found ip ${ip}`);
            await create_websocket(`ws://${ip}:64064/ws`)
            .then(cancel)
            .catch(()=>{});
            ip_search_text = ip;
          }
        }
      ) 
    }
</script>
<div class="bottom-toolbar">
  <button on:click={open_presentation_page}>Presentación</button>
  {#if opened_presentation}
  <fieldset>
    <legend align="center">Tamaño present.</legend>
    <button on:click={decrement_pres_size}>−</button>
    <button on:click={increment_pres_size}>+</button>
  </fieldset>
  {/if}
  <fieldset>
    <legend align="center">Tamaño letra</legend>
    <button on:click={decrement_font_size}>−</button>
    <span>{typeof font_size === 'string' && container? getComputedStyle(container).fontSize : trunc(font_size)+"px"}</span>
    <button on:click={increment_font_size}>+</button>
  </fieldset>
  <fieldset>
    <legend align="center">Alteraciones</legend>
    <button on:click={()=>setSharpSetting(true,container)}>#</button>
    <button on:click={()=>setSharpSetting(false,container)}>b</button>
    <button on:click={()=>setSharpSetting(null,container)}>Auto</button>
  </fieldset>
  <button on:click={(evt)=>{no_margin=!no_margin;rescroll(evt);}}>Margen</button>
  {#if connect_state == "NEW"}
  <fieldset>
    <legend align="center">Conectar</legend>
    <button on:click={()=>{connect_state = "WRITE"}}>Escribir IP</button>
    <button on:click={start_search}>Buscar IP</button>
  </fieldset>
  {:else if connect_state == "WRITE"}
  <fieldset>
    <legend align="center">Conectar</legend>
    <button on:click={()=>{connect_state = "WRITE";try_connection();}}>Conectar</button>
    <input bind:value={ip_text} type="text"/>
    <span>{connect_result_text}</span>
  </fieldset>
  {:else if connect_state == "SEARCH"}
  <fieldset>
    <legend align="center">Conectar</legend>
    <button on:click={()=>{connect_state = "WRITE";search_cancel()}}>Cancelar</button>
    <span>{ip_search_text}</span>
  </fieldset>
  {:else if connect_state == "CLOSED"}
  <button on:click={()=>{connect_state = "WRITE"}}>Conectar<br/>Presentación</button>
  {/if}
  </div>
{#if id}
  <div class="doc-id-bar" class:hidden>{id}<button on:click={edit_top}>✏️</button></div>
{/if}
<div class="view-container">
  <div
    class="inner-container u-container"
    style="--viewer-font-size:{typeof font_size === 'string'
      ? font_size
      : trunc(font_size).toString() + 'px'}"
    class:hidden
    bind:this={container}
    data-nomargin={no_margin?true:null}
  >
    {@html content}
  </div>
</div>
<svelte:document on:keydown2={keypress}/>
<style>
  .bottom-toolbar{
    display: flex;
    flex-direction: row;
    justify-content: center;
    flex-wrap: wrap;
    padding: 10px;
  }
  .hidden {
    display: none;
  }
  .inner-container {
    font-size: var(--viewer-font-size);
  }
  fieldset{
    padding: 5px;
    border-radius: 5px;
    display: inline-block;
    background-color: var(--bgcolor);
    text-align: center;
  }
  legend{
    line-height: 1;
  }
  .doc-id-bar{
    text-align: center;
    border-top: 1px solid;
  }
</style>
