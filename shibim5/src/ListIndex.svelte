<script>
    import {createEventDispatcher} from 'svelte'
    const dispatch = createEventDispatcher();
    export let lists = []
    export let hidden = false;
</script>
<div class="nav" class:hidden={hidden}>
    <button on:click={()=>dispatch("prev_page")}>Anterior</button>
    <button on:click={()=>dispatch("next_page")}>Siguiente</button>
</div>
<ul class="listindex" class:hidden={hidden}>
    {#each lists as list }
        <li>
            <button class="lname"  on:click={()=>dispatch("open_list",{name : list.name})}>
                {list.name}
            </button>
            <button class="edit" on:click={()=>dispatch("edit_list",{name : list.name})}>✏️</button>
            <ul>
                {#each list.songs as song}
                    <li>
                        {song}
                    </li>
                {/each}
            </ul>
        </li>
    {/each}
</ul>
<div class="nav" class:hidden={hidden}>
    <button on:click={()=>dispatch("prev_page")}>Anterior</button>
    <button on:click={()=>dispatch("next_page")}>Siguiente</button>
</div>
<style>
    .listindex{
        margin : 0;
        padding: 0px;
        position: relative;
        display: grid;
        grid-template-columns: repeat(auto-fill, 20em);
        gap: 10px;
        justify-content: center;
        text-align: left;
    }
    .listindex li{
        display: block;
    }
    .listindex > li{
        border-radius: 8px;
        background-color: #FEFEFE;
        box-shadow: 1px 2px 0 0 #AAA;
        padding: 4px 4px 4px 10px;
        border: 1px solid #CCC;
    }
    .listindex > li > ul{
        font-size: 12px;
        line-height: 1;
    }
    .hidden{
        display: none;
    }
    .lname{
        display: inline-block;
        margin: auto;
        font-size: 18px;
        padding: .3em .6em;
        color: #2c2cca;
    }
    .edit{
        padding: 0;
        border: none;
    }
    .nav{
        margin:5px;
        text-align: center;
    }
    @media (prefers-color-scheme: dark) {
        .listindex > li{
            border-radius: 8px;
            background-color: #000000;
            box-shadow: 1px 2px 0 0 #222;
            padding: 4px 4px 4px 10px;
            border: 1px solid #444;
        }
        .lname{
            color : #8bb8ff;
        }
    }
</style>