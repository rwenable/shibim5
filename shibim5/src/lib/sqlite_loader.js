import urldata from "./sqlite3.js?url"
import "./sqlite3-worker1-promiser"
let worker_test = `
console.debug("Loading sqlite3");
importScripts("${urldata}");
console.debug("Init sqlite3");
sqlite3InitModule().then(sqlite3 => {
  console.debug("Loading sqlite3 worker API");
  sqlite3.initWorker1API()
});`;
let blob = new Blob([worker_test], {type: 'application/javascript'});
export async function load_sqlite(){
    return new Promise((resolve,reject)=>{
      const worker = new Worker(URL.createObjectURL(blob));
      //Why, firefox, why.
      window._gclock_sql_worker = worker;
      const promiser = sqlite3Worker1Promiser({
          worker : () => worker,
          onready : () => resolve ({worker : worker, promiser : promiser}),
          onerror : (err) => reject(err)
        });
    });
}