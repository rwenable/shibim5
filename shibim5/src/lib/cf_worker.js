
import { Buffer } from "node:buffer";

export default {
    async fetch(request, env) {
      let url = new URL(request.url);
      if(url.pathname === '/kvapi/put'){
        if (!request.body){
          return new Response(null,{status : 400})
        }
        const authorization = request.headers.get("Authorization");
        if (!authorization) {
          return new Response("Missing password.", {
            status: 401
          });
        }

        const [scheme, encoded] = authorization.split(" ");
        if (!encoded || scheme !== "Basic") {
          return new Response("Malformed authorization header.", {
            status: 400,
          });
        }

        const credentials = Buffer.from(encoded, "base64").toString();

        const index = credentials.indexOf(":");
        const user = credentials.substring(0, index);
        const pass = credentials.substring(index + 1);

        if (pass === env.SECRET){
          const now = new Date();
          const ab = await request.arrayBuffer();
          const key = "db-"+now.getFullYear()+"-"+("0"+(now.getMonth()+1).toString()).slice(-2)+"-"+("0"+now.getDay().toString()).slice(-2);
          await env.SHB.put(key,ab,{expirationTtl: 60*60*24*90});
          await env.SHB.put("db-latest",ab);
          return new Response(key);
        }else{
          return new Response("Incorrect credentials.",{status : 401});
        }

      }

      if(url.pathname === '/kvapi/get_latest'){
        let content = await env.SHB.get("db-latest",{ type: "arrayBuffer" });
        if (content === null) {
          return new Response("Not found", {status: 404});
        }
        return new Response(content,{
          headers : {
            "Content-Type" : "application/octet-stream"
          }
        });
      }
       
      if (url.pathname === "/"){
        url.pathname = "/index";
        env.ASSETS.fetch(new Request(url,request));
      }
      // Otherwise, serve the static assets.
      // Without this, the Worker will error and no assets will be served.
      return env.ASSETS.fetch(request);
    },
  }
  