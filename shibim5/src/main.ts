import './app.css'
import './song.css'
import './fonts.css'
import './lib/bsc_helper'
import App from './App.svelte'
let boot_msg = document.getElementById("boot-msg");
if(boot_msg){
  boot_msg.remove();
}
const app = new App({
  target : document.getElementById('app'),
})

export default app
