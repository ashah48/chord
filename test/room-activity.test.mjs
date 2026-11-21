import test from 'node:test';
import assert from 'node:assert/strict';
import {MatrixProvider} from '../matrix-provider.mjs';

test('bridge-resent room state does not make an idle room look active',async()=>{
 const p=new MatrixProvider({}, {userId:'@me'},{save:false});
 let events=[{type:'m.room.message',event_id:'$m',origin_server_ts:1000}];
 p.raw=async()=>({rooms:{join:{'!dm':{timeline:{events}}}}});
 await p.refreshRoomsNow();
 assert.equal(p.chats.get('!dm').lastActivity,new Date(1000).toISOString());
 events=[
  {type:'m.room.name',state_key:'',origin_server_ts:9000,content:{name:'Kevin'}},
  {type:'m.bridge',state_key:'slack',origin_server_ts:9001,content:{}},
  {type:'m.room.member',state_key:'@slack_kevin',origin_server_ts:9002,content:{membership:'join'}},
  {type:'m.reaction',origin_server_ts:9003,content:{}},
  {type:'com.beeper.message_send_status',origin_server_ts:9004,content:{}},
 ];
 await p.refreshRoomsNow();
 assert.equal(p.chats.get('!dm').lastActivity,new Date(1000).toISOString());
 events=[{type:'m.room.encrypted',event_id:'$n',origin_server_ts:5000}];
 await p.refreshRoomsNow();
 assert.equal(p.chats.get('!dm').lastActivity,new Date(5000).toISOString());
});

test('saved rooms dated by old room state are re-dated once from their newest message',async()=>{
 const p=new MatrixProvider({}, {userId:'@me'},{save:false,archive:{latestMessageAt:id=>id==='matrix:!archived'?2000:0}});
 const bogus=new Date(9000).toISOString();
 p.chats.set('!cached',{id:'!cached',lastActivity:bogus});
 p.timelines.set('!cached',[{id:'$a',timestamp:new Date(1000).toISOString()}]);
 p.chats.set('!archived',{id:'!archived',lastActivity:bogus});
 p.chats.set('!server',{id:'!server',lastActivity:bogus});
 p.chats.set('!unknown',{id:'!unknown',lastActivity:bogus});
 const asked=[];
 p.raw=async(method,url,body,query)=>{asked.push(JSON.parse(query.filter).types);return {chunk:url.includes('!server')?[{origin_server_ts:3000}]:[]}};
 p.chats.set('!done',{id:'!done',lastActivity:bogus,activityVersion:4});
 await p.repairActivity();
 assert.equal(p.chats.get('!cached').lastActivity,new Date(1000).toISOString());
 assert.equal(p.chats.get('!archived').lastActivity,new Date(2000).toISOString());
 assert.equal(p.chats.get('!server').lastActivity,new Date(3000).toISOString());
 assert.equal(p.chats.get('!unknown').lastActivity,bogus);
 assert.deepEqual(asked,[['m.room.message','m.room.encrypted','m.sticker'],['m.room.message','m.room.encrypted','m.sticker']]);
 assert.equal(p.chats.get('!done').lastActivity,bogus);
 assert.ok([...p.chats.values()].every(c=>c.activityVersion===4));
});

test('activity repair retries temporary failures and ignores cached thread replies',async()=>{
 const p=new MatrixProvider({}, {userId:'@me'},{save:false});
 p.chats.set('!room',{lastActivity:new Date(9000).toISOString()});
 p.timelines.set('!room',[{id:'$reply',threadRootID:'$parent',timestamp:new Date(8000).toISOString()}]);
 p.raw=async()=>{throw Error('temporary failure')};
 await p.repairActivity();
 assert.equal(p.chats.get('!room').activityVersion,undefined);
 p.raw=async()=>({chunk:[{type:'m.room.message',origin_server_ts:1000}]});
 await p.repairActivity();
 assert.equal(p.chats.get('!room').lastActivity,new Date(1000).toISOString());
});

test('activity repair resumes after a batch full of thread replies',async()=>{
 const p=new MatrixProvider({}, {userId:'@me'},{save:false});
 p.chats.set('!room',{lastActivity:new Date(9000).toISOString(),activityVersion:3});
 const cursors=[];
 p.raw=async(method,url,body,query)=>{
  const cursor=Number(query.from)||0;
  cursors.push(cursor);
  return cursor===3?{chunk:[{type:'m.room.message',origin_server_ts:1000}]}:
   {chunk:[{type:'m.room.message',origin_server_ts:8000,content:{'m.relates_to':{rel_type:'m.thread'}}}],end:String(cursor+1)};
 };
 await p.repairActivity();
 const saved=JSON.parse(JSON.stringify(p.chats.get('!room')));
 assert.equal(saved.activityRepairCursor,'3');
 assert.equal(saved.activityVersion,3);
 const resumed=new MatrixProvider({}, {userId:'@me'},{save:false});
 resumed.chats.set('!room',saved);
 resumed.raw=p.raw;
 await resumed.repairActivity();
 assert.deepEqual(cursors,[0,1,2,3]);
 assert.equal(resumed.chats.get('!room').lastActivity,new Date(1000).toISOString());
 assert.equal(resumed.chats.get('!room').activityRepairCursor,undefined);
 assert.equal(resumed.chats.get('!room').activityVersion,4);
});

test('Matrix inbox pages advance independently of pinned rows',async()=>{
 const p=new MatrixProvider({}, {userId:'@me'},{save:false});
 p.lastRoomRefresh=Date.now();
 for(let i=1;i<=4;i++)p.chats.set('!'+i,{id:'!'+i,accountID:'slack',lastActivity:new Date(i*1000).toISOString()});
 const first=await p.request('/v1/chats?limit=2&accountIDs=slack&includeRoom=!1');
 assert.deepEqual(first.items.map(x=>x.id),['!4','!3','!1']);
 assert.deepEqual(JSON.parse(first.oldestCursor),[new Date(3000).toISOString(),'!3']);
 p.chats.set('!5',{id:'!5',accountID:'slack',lastActivity:new Date(5000).toISOString()});
 const next=await p.request('/v1/chats?limit=2&accountIDs=slack&cursor='+encodeURIComponent(first.oldestCursor)+'&includeRoom=!1');
 assert.deepEqual(next.items.map(x=>x.id),['!2','!1']);
 assert.equal(next.hasMore,false);
});
