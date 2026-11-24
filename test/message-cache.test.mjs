import test from 'node:test';
import assert from 'node:assert/strict';
import {MatrixProvider,reactionRecord} from '../matrix-provider.mjs';
test('history returns while reaction recovery is stalled and duplicate page requests share work',async()=>{
 let finishHistory,finishRelations,calls=0;
 const p=new MatrixProvider({messages:{list:()=>{calls++;return new Promise(r=>finishHistory=r)}}},{userId:'@me:chord.local'},{save:false});
 p.chats.set('!room',{participants:{items:[]},accountID:'whatsapp'});
 p.raw=()=>new Promise(r=>finishRelations=r);
 const first=p.listMessages('!room'),second=p.listMessages('!room');
 assert.equal(calls,1);finishHistory({messages:[{roomId:'!room',eventId:'$one',text:'Fast text',sender:{userId:'@other:chord.local'},timestamp:1}],nextCursor:'older'});
 const [a,b]=await Promise.all([first,second]);assert.equal(a.items[0].text,'Fast text');assert.deepEqual(a,b);assert.ok(p.relationRefresh.has('!room|$one'));
 finishRelations({chunk:[]});await p.relationRefresh.get('!room|$one');
});
test('confirmed reactions update immediately and a stale recovery cannot undo them',async()=>{
 let release;
 const p=new MatrixProvider({reactions:{send:async()=>({eventId:'$reaction'}),redact:async()=>({})}},{userId:'@me:chord.local'},{save:false});
 p.raw=()=>new Promise(r=>release=r);
 const recovery=p.refreshReactions('!room',[{id:'$message'}]);
 await p.request('/v1/chats/'+encodeURIComponent('!room')+'/messages/'+encodeURIComponent('$message')+'/reactions','POST',{reactionKey:'👍'});
 release({chunk:[]});await recovery;
 assert.equal(p.reactions.get('!room|$message')[0].reactionKey,'👍');
 await p.request('/v1/chats/'+encodeURIComponent('!room')+'/messages/'+encodeURIComponent('$message')+'/reactions/'+encodeURIComponent('👍'),'DELETE');
 assert.equal(p.reactions.get('!room|$message').length,0);
});
test('conversation activity follows its newest message despite an older room snapshot',async()=>{
 const p=new MatrixProvider({}, {userId:'@me:chord.local'}, {save:false});
 p.lastRoomRefresh=Date.now();
 p.chats.set('!room',{id:'!room',accountID:'discord',title:'Important',lastActivity:'2026-11-09T16:56:48Z'});
 p.timelines.set('!room',[{id:'$new',timestamp:'2026-11-10T01:51:07Z',text:'New message',senderName:'Sam'}]);
 const result=await p.request('/v1/chats');
 assert.equal(result.items[0].lastActivity,'2026-11-10T01:51:07.000Z');
 assert.equal(result.items[0].preview.text,'New message');
 assert.equal(result.items[0].preview.senderName,'Sam');
});
test('thread replies stay out of the sidebar preview and do not sort the channel',async()=>{
 const p=new MatrixProvider({}, {userId:'@me:chord.local'}, {save:false});
 p.lastRoomRefresh=Date.now();
 p.chats.set('!room',{id:'!room',accountID:'discord',title:'General',type:'group',lastActivity:'2026-11-09T16:56:48.000Z',preview:{text:'Channel note',senderName:'Ada',senderID:'@ada'}});
 p.timelines.set('!room',[
  {id:'$main',timestamp:'2026-11-10T01:00:00.000Z',text:'Channel note',senderName:'Ada',senderID:'@ada',isSender:true},
  {id:'$reply',timestamp:'2026-11-10T03:00:00.000Z',text:'Inside the thread',senderName:'Pat',senderID:'@pat',threadRootID:'$main',isSender:false,isMention:true,linkedMessageID:'$main'},
 ]);
 const result=await p.request('/v1/chats');
 assert.equal(result.items[0].preview.text,'Channel note');
 assert.equal(result.items[0].preview.senderName,'Ada');
 assert.equal(result.items[0].lastActivity,'2026-11-10T01:00:00.000Z');
 assert.equal(result.items[0].signals.mention,false);
 assert.equal(result.items[0].signals.replyToYou,false);
});
test('a live thread reply does not replace the channel preview',()=>{
 const p=new MatrixProvider({}, {userId:'@me:chord.local'}, {save:false});
 p.chats.set('!room',{id:'!room',preview:{text:'Main line',senderName:'Ada',senderID:'@ada'},lastActivity:'2026-11-01T00:00:00.000Z',participants:{items:[]}});
 p.onEvent({kind:'message',roomId:'!room',eventId:'$thread',text:'Thread only',timestamp:Date.parse('2026-11-18T00:00:00Z'),sender:{userId:'@other:chord.local'},threadRoot:'$root'});
 assert.equal(p.chats.get('!room').preview.text,'Main line');
 assert.equal(p.chats.get('!room').lastActivity,'2026-11-01T00:00:00.000Z');
 assert.equal(p.timelines.get('!room')[0].threadRootID,'$root');
 p.onEvent({kind:'message',roomId:'!room',eventId:'$main',text:'On the channel',timestamp:Date.parse('2026-11-18T01:00:00Z'),sender:{userId:'@other:chord.local',isMe:false}});
 assert.equal(p.chats.get('!room').preview.text,'On the channel');
});
test('discord and slack custom reactions keep their image and shortcode',()=>{
 const discord=reactionRecord({eventId:'$r',sender:'@discord_1:chord.local',key:'mxc://chord.local/blob',content:{'com.beeper.reaction.shortcode':':blobcat:','fi.mau.discord.reaction':{id:'1',name:'blobcat',mxc:'mxc://chord.local/blob'},'m.relates_to':{rel_type:'m.annotation',key:'mxc://chord.local/blob'}}});
 assert.equal(discord.reactionKey,'mxc://chord.local/blob');
 assert.equal(discord.image,'mxc://chord.local/blob');
 assert.equal(discord.shortcode,':blobcat:');
 assert.equal(discord.participantID,'@discord_1:chord.local');
 const slack=reactionRecord({id:'$s',sender:'@slack_1:chord.local',content:{'fi.mau.slack.reaction':{name:'harrisun',mxc:'mxc://chord.local/sun'},'m.relates_to':{key:'mxc://chord.local/sun'}}});
 assert.equal(slack.shortcode,':harrisun:');
 assert.equal(slack.image,'mxc://chord.local/sun');
});
