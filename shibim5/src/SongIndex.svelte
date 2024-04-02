<script>
    import {createEventDispatcher} from 'svelte';
    import {NT_s} from './lib/song';
    const dispatch = createEventDispatcher();
    export let songs = [];
    export let hidden = true;
</script>
<div class="nav" class:hidden={hidden}>
    <button on:click={()=>dispatch("prev_page")}>Anterior</button>
    <button on:click={()=>dispatch("next_page")}>Siguiente</button>
</div>
<ul class="songindex" class:hidden={hidden}>
    {#each songs as song }
        <li>
            <button class="sname" on:click={()=>dispatch("open_song",{name:song.name})}>{song.title}
            </button>
            <button class="edit" on:click={()=>dispatch("edit_song",{name : song.name})}>✏️</button>
            {#if song.subtitle}
                <span class="ssubtitle">{song.subtitle}
                    <span class="tonality">({@html NT_s[song.tonic]}{#if song.tonic_kind === 1}m{/if})</span>
                </span>
            {:else}
                <span class="ssubtitle">{song.name}
                    <span class="tonality">({@html NT_s[song.tonic]}{#if song.tonic_kind === 1}m{/if})</span>
                </span>
            {/if}
            
            <div class="section-box">
                {#each song.sections as section}
                    <span class="section">{section.replace("|"," ")}</span>
                {/each}
            </div>
        </li>
    {/each}
</ul>
<div class="nav" class:hidden={hidden}>
    <button on:click={()=>dispatch("prev_page")}>Anterior</button>
    <button on:click={()=>dispatch("next_page")}>Siguiente</button>
</div>
<style>
    .sname{
        color: #2c2cca;
        padding: 0;
    }
    .nav{
        margin:5px;
        text-align: center;
    }
    .songindex{
        margin : 0;
        font-size: 18px;
        padding: 0px;
        display: grid;
        grid-template-columns: repeat(auto-fill, 20em);
        gap: 10px;
        text-align: left;
        justify-content: center;
    }
    .hidden{
        display: none;
    }
    .songindex li{
        break-inside: avoid;
        border-radius: 5px;
        background-color: var(--bgcolor);
    }
    .section{
        display: inline-block;
        margin-right: 1em;
    }
    .section-box{
        font-size: 10px;
        line-height: 1;
        padding-bottom: 3px;
    }
    .ssubtitle {
        font-size: 12px;
        display: block;
        line-height: 1;
    }
    ul{
        list-style-type: none;
    }
    .edit{
        padding: 0;
        border: none;
    }
    @media (prefers-color-scheme: dark) {
        .sname{
            color : #8bb8ff;
        }
    }
</style>