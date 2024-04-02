import { defineConfig } from 'vite'
import { resolve } from 'path';
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { viteSingleFile } from "vite-plugin-singlefile"
import handlebars from 'vite-plugin-handlebars';
// https://vitejs.dev/config/
export default defineConfig({
  plugins: [handlebars({
	  partialDirectory: resolve(__dirname, 'include')
  }),svelte(),viteSingleFile()],
  build:{
    assetsInlineLimit : 4096*1024,
    minify : false
  }
})
