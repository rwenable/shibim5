<script>
    import {createEventDispatcher} from "svelte";
    const dispatch = createEventDispatcher();
    export let results = [];
    export let hidden = true;
</script>
<input type="text" on:input class:hidden={hidden} autofocus/>
<div class="results" class:hidden={hidden}>
    {#each results as entry}
        <div class="entry">
            <button on:click={dispatch("open_song",{name : entry[1]})}>{entry[2]}</button>
            <button class="edit" on:click={()=>dispatch("edit_song",{name : entry[1]})}>✏️</button>
            <div class="content">
                {entry[6]}
            </div>
        </div>
    {/each}
</div>
<style>
    input{
        margin: 20px auto;
        display: block;
        box-sizing: border-box;
        width: 80%;

    }
    .results{
        margin : 0;
        padding: 0px;
        position: relative;
        display: grid;
        grid-template-columns: repeat(auto-fill, 20em);
        gap: 10px;
        justify-content: center;
        text-align: left;
    }
    .content{
        font-size: 12px;
        max-height: 4.9em;
        word-wrap: break-word;
        overflow: hidden;
    }
    .hidden{
        display: none;
    }
    
</style>