import test from 'node:test';
import assert from 'node:assert/strict';
import {MatrixProvider} from '../matrix-provider.mjs';
test('mark read advances unopened and partially cached rooms to the current head',async()=>{
 for(const cached of [[],[{id:'$old'}]]){
  const receipts=[];const p=new MatrixProvider({}, {userId:'@me'},{save:false});
  p.chats.set('!room',{id:'!room',unreadCount:5});p.timelines.set('!room',cached);
  p.raw=async(method,url,body,query)=>{assert.equal(query.dir,'b');assert.equal(query.limit,'1');return {chunk:[{event_id:'$head'}]}};
  p.client={accountData:{setRoom:async()=>{}},messages:{markRead:async r=>receipts.push(r)}};
  await p.request('/v1/chats/!room/read','POST',{});
  assert.deepEqual(receipts,[{roomId:'!room',eventId:'$head'}]);assert.equal(p.chats.get('!room').unreadCount,0);
 }
});
test('a read watermark survives stale server counts and permits future unread messages',async()=>{
 const p=new MatrixProvider({}, {userId:'@me'},{save:false});
 p.chats.set('!room',{id:'!room',localReadAt:2000});
 let timestamp=1000;
 p.raw=async()=>({rooms:{join:{'!room':{timeline:{events:[{event_id:'$head',origin_server_ts:timestamp}]},unread_notifications:{notification_count:3,highlight_count:1}}}}});
 await p.refreshRoomsNow();assert.equal(p.chats.get('!room').unreadCount,0);
 timestamp=3000;
 await p.refreshRoomsNow();assert.equal(p.chats.get('!room').unreadCount,3);
});
test('old highlighted pings do not turn a later ordinary message into a new ping',async()=>{
 const p=new MatrixProvider({}, {userId:'@me'},{save:false});
 p.chats.set('!room',{id:'!room',localReadAt:2000,readHighlightBaseline:2});
 let highlights=2;
 p.raw=async()=>({rooms:{join:{'!room':{timeline:{events:[{origin_server_ts:3000}]},unread_notifications:{notification_count:4,highlight_count:highlights}}}}});
 await p.refreshRoomsNow();assert.equal(p.chats.get('!room').highlightCount,0);
 highlights=3;await p.refreshRoomsNow();assert.equal(p.chats.get('!room').highlightCount,1);
 highlights=0;await p.refreshRoomsNow();assert.equal(p.chats.get('!room').readHighlightBaseline,0);
 highlights=1;await p.refreshRoomsNow();assert.equal(p.chats.get('!room').highlightCount,1);
});
test('mark read survives a concurrent room refresh replacing the conversation object',async()=>{
 let release,started;
 const waiting=new Promise(r=>started=r),gate=new Promise(r=>release=r);
 const p=new MatrixProvider({accountData:{setRoom:async()=>{started();await gate}},messages:{markRead:async()=>{}}},{userId:'@me'},{save:false});
 p.chats.set('!room',{id:'!room',unreadCount:5,highlightCount:1});
 p.raw=async(method,path)=>path.endsWith('/messages')?{chunk:[{event_id:'$head'}]}:{rooms:{join:{'!room':{timeline:{events:[{origin_server_ts:1000}]},unread_notifications:{notification_count:5,highlight_count:1}}}}};
 const reading=p.request('/v1/chats/!room/read','POST',{});await waiting;
 await p.refreshRoomsNow();release();await reading;
 assert.equal(p.chats.get('!room').unreadCount,0);assert.equal(p.chats.get('!room').isMarkedUnread,false);
 await p.refreshRoomsNow();assert.equal(p.chats.get('!room').unreadCount,0);
});
test('a thread-only sync does not unread the channel or replace its preview',async()=>{
 const p=new MatrixProvider({}, {userId:'@me'},{save:false});
 p.syncToken='s1';
 p.chats.set('!room',{id:'!room',accountID:'discord',title:'General',lastActivity:'2026-11-01T00:00:00.000Z',unreadCount:0,highlightCount:0,rawHighlightCount:0,preview:{text:'channel hello',senderName:'Ada'}});
 p.raw=async()=>({next_batch:'s2',rooms:{join:{'!room':{timeline:{events:[{type:'m.room.message',origin_server_ts:Date.parse('2026-11-18T00:00:00Z'),sender:'@other:chord.local',content:{body:'thread reply','m.relates_to':{rel_type:'m.thread',event_id:'$root'}}}]},unread_notifications:{notification_count:4,highlight_count:2}}}}});
 await p.refreshRoomsNow();
 const room=p.chats.get('!room');
 assert.equal(room.unreadCount,0);
 assert.equal(room.highlightCount,0);
 assert.equal(room.preview.text,'channel hello');
 assert.equal(room.lastActivity,'2026-11-01T00:00:00.000Z');
 p.raw=async()=>({next_batch:'s3',rooms:{join:{'!room':{timeline:{events:[{type:'m.room.message',origin_server_ts:Date.parse('2026-11-18T02:00:00Z'),sender:'@other:chord.local',content:{body:'on the channel'}}]},unread_notifications:{notification_count:1,highlight_count:0}}}}});
 await p.refreshRoomsNow();
 assert.equal(p.chats.get('!room').unreadCount,1);
 assert.equal(p.chats.get('!room').preview.text,'on the channel');
 assert.equal(p.chats.get('!room').lastActivity,new Date(Date.parse('2026-11-18T02:00:00Z')).toISOString());
});
test('thread replies stay out of the channel unread and mention counts',async()=>{
 const p=new MatrixProvider({}, {userId:'@me'},{save:false});
 let filter;
 p.raw=async(_method,_path,_body,query)=>{filter=JSON.parse(query.filter);return {rooms:{join:{'!room':{timeline:{events:[{origin_server_ts:3000,content:{'m.relates_to':{rel_type:'m.thread',event_id:'$root'}}}]},unread_notifications:{notification_count:0,highlight_count:0},unread_thread_notifications:{'$root':{notification_count:6,highlight_count:2}}}}}}};
 await p.refreshRoomsNow();
 assert.equal(filter.room.timeline.unread_thread_notifications,true);
 assert.equal(p.chats.get('!room').unreadCount,0);
 assert.equal(p.chats.get('!room').highlightCount,0);
 p.raw=async()=>({rooms:{join:{'!room':{timeline:{events:[{origin_server_ts:4000}]},unread_notifications:{notification_count:2,highlight_count:1},unread_thread_notifications:{'$root':{notification_count:6,highlight_count:2}}}}}});
 await p.refreshRoomsNow();
 assert.equal(p.chats.get('!room').unreadCount,2);
 assert.equal(p.chats.get('!room').highlightCount,1);
});
test('explicit unread stays set when a stale sync response says read',async()=>{
 const p=new MatrixProvider({accountData:{setRoom:async()=>{}}},{userId:'@me'},{save:false});p.chats.set('!room',{id:'!room'});
 p.raw=async()=>({rooms:{join:{'!room':{timeline:{events:[{origin_server_ts:1000}]},account_data:{events:[{type:'m.marked_unread',content:{unread:false}}]},unread_notifications:{notification_count:0}}}}});
 await p.request('/v1/chats/!room/unread','POST',{});await p.refreshRoomsNow();assert.equal(p.chats.get('!room').isMarkedUnread,true);
});
