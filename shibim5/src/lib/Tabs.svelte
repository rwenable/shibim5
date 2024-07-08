<script>
    import {createEventDispatcher} from 'svelte'
    export let current = "lists";
    export let tab_save_open = false;
    export let tab_upload_open = false;
    let tabs = ["lists","songs","search","editor"]
    let tabnames = ["Listas","Canciones","Busqueda","Editor"]
    const dispatch = createEventDispatcher();
    export function change_tab(name){
        dispatch("change_tab",{name:name})
        current = name;
    }
    export function change_tab_silent(name){
        current = name;
    }
</script>
<nav>
    {#each tabs as tab, i }
            <button on:click={()=>change_tab(tab)}>
                {tabnames[i]}
            </button>
    {/each}
    <button on:click={()=>dispatch("download_app",{})}>Descargar</button>
    <button on:click={()=>dispatch("upload_app",{})}>Subir</button>
    </nav>
<main>
    {#if tab_save_open}
        <slot name="download"/>
    {/if}
    {#if tab_upload_open}
        <slot name="upload"/>
    {/if}
    {#if current == tabs[0]}
        <slot name="lists"/>
    {:else if current == tabs[1]}
        <slot name="songs"/>
    {:else if current == tabs[2]}    
        <slot name="search"/>
    {/if}
    <div class:hidden={current!=tabs[3]}>
        <slot name="editor"/>
    </div>
</main>
<style>
nav {
    text-align: center;
    background-color: #FFF;
    --grad-color: rgba(155, 155, 155, 0.15);
    background-image: 
    repeating-linear-gradient(
        135deg,
        transparent,
        transparent 3px,
        var(--grad-color) 4px,
        var(--grad-color)6px,
        transparent 7px,
        transparent 10px ),
    repeating-linear-gradient(
        195deg,
        transparent,
        transparent 3px,
        var(--grad-color) 4px,
        var(--grad-color) 6px,
        transparent 7px,
        transparent 10px ),
    repeating-linear-gradient(
        255deg,
        transparent,
        transparent 3px,
        var(--grad-color) 4px,
        var(--grad-color) 6px,
        transparent 7px,
        transparent 10px ),
    repeating-linear-gradient(
        135deg,
        transparent,
        transparent 1.5px,
        rgba(128,128,128,0.1) 2px,
        rgba(128,128,128,0.1) 3px,
        transparent 3.5px,
        transparent 5px ),
    repeating-linear-gradient(
        195deg,
        transparent,
        transparent 1.5px,
        rgba(128,128,128,0.1) 2px,
        rgba(128,128,128,0.1) 3px,
        transparent 3.5px,
        transparent 5px ),
    repeating-linear-gradient(
        255deg,
        transparent,
        transparent 1.5px,
        rgba(128,128,128,0.1) 2px,
        rgba(128,128,128,0.1) 3px,
        transparent 3.5px,
        transparent 5px );
    border-bottom: 1px solid var(--separator-color);
}
.hidden{
    display: none;
}
@media (prefers-color-scheme: dark) {
    nav{
        background-color: #000;
    }
}
</style>