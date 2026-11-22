// Run after building Pickle. Uses the real WASM runtime and synthetic data only.
import assert from 'node:assert/strict';
import {setTimeout as delay} from 'node:timers/promises';
import {createMatrixClient} from '../packages/pickle/dist/node.js';

let active=0,peak=0,lookups=0;
const events=Array.from({length:12},(_,i)=>({event_id:`$${i}`,type:'m.room.message',sender:'@alice:example',origin_server_ts:i+1,content:{msgtype:'m.text',body:`message ${i}`}}));
const client=createMatrixClient({
 account:{homeserver:'https://synthetic.invalid',userId:'@me:example',deviceId:'TEST',accessToken:'synthetic'},
 pickleKey:'synthetic',
 fetch:async input=>{
  const url=new URL(input);
  let body={};
  if(url.pathname.endsWith('/messages'))body={chunk:events,end:'older'};
  if(url.pathname.includes('/relations/')) {
   lookups++;
   peak=Math.max(peak,++active);
   try {await delay(5)} finally {active--}
   body={chunk:[]};
  }
  return Response.json(body);
 },
});
try {
 const page=await client.messages.list({roomId:'!room:example',limit:12});
 assert.equal(page.messages.length,12);
 assert.equal(page.nextCursor,'older');
 assert.equal(lookups,12);
 assert.equal(peak,4);
 assert.deepEqual(page.messages.map(m=>m.eventId),events.map(e=>e.event_id));
 console.log('WASM history smoke passed: 12 messages, 4 concurrent edit lookups, order and cursor preserved.');
} finally {await client.close()}
