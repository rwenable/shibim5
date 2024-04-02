<script>
    export let progress = 0;
</script>
<div class="progress">
    <span style="width: {progress}%"></span>
</div>
<style>
    .progress { 
      height: 20px;
      position: relative;
      background: var(--bgcolor);
      border-radius: 25px;
      padding: 10px;
      box-shadow: inset 0 0 3px var(--main-font-color);
      margin: 10px;
    }
    .progress > span {
      display: block;
      height: 100%;
      border-radius: 20px;
      background-color: var(--emphasis3-color);
      box-shadow: 
        inset 0 2px 9px  rgba(255,255,255,0.3),
        inset 0 -2px 6px rgba(0,0,0,0.4);
      position: relative;
      overflow: hidden;
    }
  </style>