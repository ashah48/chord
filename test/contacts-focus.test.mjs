import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp} from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {ContactBook} from '../contacts.mjs';
import {Directory,messageEmotes} from '../directory.mjs';
import {applyLocalRules,importance,rankInbox} from '../importance.mjs';

const memoryBook=()=>{const book=new ContactBook('/unused');book.save=async()=>{};return book};

test('matching names do not link people, exact verified phone identifiers do',()=>{
 const book=memoryBook();
 const alice=book.observe({key:'@discord_1:chord.local',name:'Alex',source:'discord'});
 const another=book.observe({key:'@discord_2:chord.local',name:'Alex',source:'discord'});
 assert.notEqual(alice.id,another.id);
 book.observe({key:'tel:+14155550100',name:'Alex Chen',source:'iphone',rank:100});
 book.observe({key:'@whatsapp_14155550100:chord.local',aliases:['tel:+14155550100'],name:'Alex',source:'whatsapp'});
 assert.equal(book.person('tel:+14155550100').id,book.person('@whatsapp_14155550100:chord.local').id);
 assert.equal(book.name('@whatsapp_14155550100:chord.local'),'Alex Chen');
 assert.equal(book.name('@discord_2:chord.local'),'Alex');
});

test('names and favorites persist and survive later network-profile updates',async()=>{
 const dir=await mkdtemp(path.join(os.tmpdir(),'chord-contact-test-'));
 const book=await new ContactBook(dir).load();
 const person=book.observe({key:'@slack_t1-u1:chord.local',name:'Alex',source:'slack'});
 await book.update(person.id,{name:'Alex from lab',favorite:true});
 const loaded=await new ContactBook(dir).load();
 loaded.observe({key:'@slack_t1-u1:chord.local',name:'Alexander',source:'slack',rank:60});
 assert.equal(loaded.name('@slack_t1-u1:chord.local'),'Alex from lab');
 assert.equal(loaded.person('@slack_t1-u1:chord.local').favorite,true);
});

test('explicit contact links can be undone without losing either record',async()=>{
 const book=memoryBook();
 const a=book.observe({key:'@slack_t1-u1:chord.local',name:'Alex',source:'slack'});
 const b=book.observe({key:'@discord_1:chord.local',name:'Alex',source:'discord'});
 await book.update(a.id,{name:'Alex Chen',favorite:true});
 await book.merge(a.id,b.id);
 assert.equal(book.person('@slack_t1-u1:chord.local').id,b.id);
 await book.undoMerge();
 assert.equal(book.person('@slack_t1-u1:chord.local').id,a.id);
 assert.equal(book.name('@slack_t1-u1:chord.local'),'Alex Chen');
 assert.equal(book.person('@discord_1:chord.local').id,b.id);
});

test('reaction names use contacts, call you You, and reuse senders on the page',async()=>{
 const book=memoryBook();
 book.observe({key:'@slack_t1-u1:chord.local',name:'Alex',source:'slack'});
 const directory=new Directory(book,{blue:async()=>[],config:()=>({})});
 directory.catalog.selfIds.add('@max:chord.local');
 const [first,second]=await directory.enrichMessages([
  {id:'a',senderID:'@slack_t1-u9:chord.local',senderName:'Pat',text:'Earlier',reactions:[{reactionKey:'👍',participantID:'@slack_t1-u1:chord.local'},{reactionKey:'👍',participantID:'@max:chord.local'},{reactionKey:'👍',participantID:'@slack_t1-u2:chord.local'}]},
  {id:'b',senderID:'@slack_t1-u2:chord.local',senderName:'Sam',text:'Later',reactions:[]},
 ],{source:'slack'});
 assert.equal(first.reactions[0].participantName,'Alex');
 assert.equal(first.reactions[1].participantName,'You');
 assert.equal(first.reactions[2].participantName,'Sam');
 assert.equal(second.senderName,'Sam');
});

test('slack custom emoji in a message point at that workspace image',async()=>{
 const directory=new Directory(memoryBook(),{blue:async()=>[],config:()=>({})});
 directory.catalog.slackEmoji=new Map([['TSAU3JLKZ',new Map([['boomparrot','mxc://chord.local/parrot']])]]);
 const [m]=await directory.enrichMessages([
  {senderID:'@slack_t1-u1:chord.local',senderName:'Vyju',text:':boomparrot:wawawa and 7:30-4:00'},
 ],{source:'slack',workspaceID:'TSAU3JLKZ'});
 assert.equal(m.text,':boomparrot:wawawa and 7:30-4:00');
 assert.deepEqual(m.emotes,[{name:'boomparrot',mxc:'mxc://chord.local/parrot'}]);
 const [other]=await directory.enrichMessages([{text:':boomparrot:'}],{source:'slack',workspaceID:'TEMMBGZU3'});
 assert.equal(other.emotes,undefined);
 assert.deepEqual(messageEmotes(':boomparrot::party_parrot:',directory.catalog.slackEmoji.get('TSAU3JLKZ')),[{name:'boomparrot',mxc:'mxc://chord.local/parrot'}]);
});

test('message senders use contact names and local renames across linked accounts',async()=>{
 const book=memoryBook();const person=book.observe({key:'@slack_t1-u1:chord.local',name:'Alex',source:'slack'});
 await book.update(person.id,{name:'Alex from lab'});
 const directory=new Directory(book,{blue:async()=>[],config:()=>({})});
 const [m]=await directory.enrichMessages([{senderID:'@slack_t1-u1:chord.local',senderName:'@slack_t1-u1:chord.local',text:'Hello'}],{source:'slack'});
 assert.equal(m.senderName,'Alex from lab');assert.equal(m.senderContactID,person.id);
});

test('only a manual mark makes a conversation Important',()=>{
 const chat={direct:true,unread:true,signals:{mention:true},attention:{important:true}};
 assert.equal(importance(chat).important,false);
 assert.equal(importance(chat,{preference:{priority:'important',muted:true}}).important,true);
});

test('important overrides persist, the retired quiet rule reads as automatic, All contains every conversation',async()=>{
 const book=memoryBook();const now=Date.now(),timestamp=new Date(now).toISOString();
 const conversations=[{id:'chat:one',timestamp,direct:false},{id:'chat:two',timestamp,direct:false}];
 await assert.rejects(()=>book.setPreference('chat:one',{priority:'quiet'}),/auto, important, or unimportant/);
 book.data.conversations['chat:one']={priority:'quiet'};
 assert.equal(book.preference('chat:one').priority,'auto');
 await book.setPreference('chat:two',{priority:'important'});
 const focus=rankInbox(conversations,book,{view:'focus',now});
 assert.deepEqual(focus.items.map(c=>c.id),['chat:two']);assert.equal(focus.summary.hidden,1);
 assert.equal(focus.items[0].importance.reasons[0],'Marked important by you');
 assert.equal(rankInbox(conversations,book,{view:'all',now}).items.length,2);
 const ruled=applyLocalRules([{id:'chat:one',timestamp,unread:true,unreadCount:4}],book,now);
 assert.equal(ruled[0].unread,true);
 assert.equal(ruled[0].unreadCount,4);
 await book.setPreference('chat:two',{priority:'auto'});
 assert.equal(rankInbox(conversations,book,{view:'focus',now}).items.length,0);
});
test('a mute lasts for one of five durations, hides unread, and keeps the priority underneath',async()=>{
 const book=memoryBook();
 const now=Date.parse('2026-11-12T15:00:00Z');
 const chat={id:'chat:one',timestamp:new Date(now).toISOString(),direct:true,unread:true};
 await assert.rejects(()=>book.setPreference('chat:one',{muteMinutes:90},now),/1 hour, 4 hours, 24 hours, 3 days, or 7 days/);
 for(const minutes of [60,240,1440,4320,10080]){
  const saved=await book.setPreference('chat:one',{priority:'important',muteMinutes:minutes},now);
  assert.equal(saved.priority,'important');
  assert.equal(saved.muted,true);
  assert.equal(saved.muteUntil,now+minutes*60000);
 }
 const hidden=applyLocalRules([chat],book,now);
 assert.equal(hidden[0].unread,false);
 assert.equal(hidden[0].sourceUnread,true);
 assert.equal(hidden[0].localPriority,'important');
 assert.equal(rankInbox(hidden,book,{view:'focus',now}).items[0].unread,false);
 const later=now+10080*60000+1;
 assert.equal(book.preference('chat:one',later).muted,false);
 const back=applyLocalRules([{...chat,unread:true}],book,later);
 assert.equal(back[0].unread,true);
 const stuck=applyLocalRules([{...chat,unread:true,isMuted:true,muteUntil:now}],book,later);
 assert.equal(stuck[0].isMuted,false);
 assert.equal(stuck[0].unread,true);
 assert.equal(rankInbox(back,book,{view:'focus',now:later}).items.map(c=>c.id).includes('chat:one'),true);
 await book.setPreference('chat:one',{muteMinutes:60},now);
 await book.setPreference('chat:one',{muteMinutes:null},now);
 assert.equal(book.preference('chat:one',now).muted,false);
 assert.equal(book.preference('chat:one',now).priority,'important');
});
test('direct conversations choose the other person for photos, excluding bridge bots and self',()=>{
 const book=memoryBook();
 const bot=book.observe({key:'@slackbot:chord.local',name:'Chord Slack'});
 const jiao=book.observe({key:'@slack_team-jiao:chord.local',name:'Jiao Deng',source:'slack'});
 const me=book.observe({key:'@max:chord.local',name:'Max'});
 const other=book.observe({key:'@slack_team-other:chord.local',name:'Other participant',source:'slack'});
 const directory=new Directory(book,{blue:async()=>[],config:()=>({})});
 directory.catalog.selfIds=new Set(['@max:chord.local']);
 directory.catalog.rooms.set('!dm',{network:'slack',direct:true,otherUser:'@slack_team-jiao:chord.local'});
 const conversation=directory.enrichConversation({id:'matrix:!dm',remoteId:'!dm',kind:'chat',source:'slack',participants:['@slackbot:chord.local','@max:chord.local','@slack_team-other:chord.local','@slack_team-jiao:chord.local']});
 assert.equal(conversation.title,'Jiao Deng');assert.equal(conversation.contactIDs[0],jiao.id);
 assert.equal(conversation.contactIDs.includes(bot.id),false);assert.equal(conversation.contactIDs.includes(me.id),false);assert.equal(conversation.contactIDs.includes(other.id),true);
});
test('composer details retain the room nickname, other names, and cached custom emoji',()=>{
 const book=memoryBook(),id='@discord_123:chord.local';
 const person=book.observe({key:id,name:'Victoria',handle:'victoria23'});
 person.customName='Vicky';
 const directory=new Directory(book,{blue:async()=>[],config:()=>({})});
 directory.catalog.rooms.set('!room',{network:'discord',serverID:'server'});
 directory.catalog.discordEmoji=new Map([['server',new Map([['party','mxc://chord.local/party']])]]);
 const details=directory.composerDetails('!room',{participants:{items:[{id,fullName:'Victoria B'}]}},{[id]:'Vic'});
 assert.equal(details.participants.items[0].fullName,'Vic');
 assert.deepEqual(details.participants.items[0].aliases,['Victoria B','Vicky','Victoria','victoria23']);
 assert.deepEqual(details.customEmoji,[{name:'party',mxc:'mxc://chord.local/party'}]);
});

test('the self profile follows the conversation network and workspace',()=>{
 const directory=new Directory(memoryBook(),{blue:async()=>[],config:()=>({})});
 directory.catalog.selfIds=new Set([
  '@max:chord.local','@discord_123:chord.local','@slack_tone-u1:chord.local',
  '@slack_ttwo-u2:chord.local','@slack_ttwo-u3:chord.local',
 ]);
 const rooms=directory.catalog.rooms;
 rooms.set('!buggy',{network:'discord',serverID:'buggy'});
 rooms.set('!lab',{network:'slack',workspaceID:'TONE'});
 rooms.set('!class',{network:'slack',workspaceID:'TTWO',receiver:'TTWO-U3'});
 rooms.set('!ambiguous',{network:'slack',workspaceID:'TTWO'});
 assert.equal(directory.selfProfileIdentity('!buggy'),'@discord_123:chord.local');
 assert.equal(directory.selfProfileIdentity('!lab'),'@slack_tone-u1:chord.local');
 assert.equal(directory.selfProfileIdentity('!class'),'@slack_ttwo-u3:chord.local');
 assert.equal(directory.selfProfileIdentity('!ambiguous'),null);
});

test('inbox previews say You for the signed-in account and its bridge identities',()=>{
 const book=memoryBook(),owner='@max:chord.local',puppet='@discord_123:chord.local',other='@discord_456:chord.local';
 const directory=new Directory(book,{blue:async()=>[],config:()=>({matrixAccount:{userId:owner}})});
 directory.catalog.selfIds.add(puppet);
 for(const id of [owner,puppet,other])book.observe({key:id,name:'max'});
 const conversation={id:'matrix:!dm',remoteId:'!dm',kind:'chat',source:'discord'};
 for(const id of [owner,puppet])assert.equal(directory.enrichConversation({...conversation,previewSenderID:id}).previewSenderName,'You');
 assert.equal(directory.enrichConversation({...conversation,previewSenderName:'max',previewIsSender:true}).previewSenderName,'You');
 assert.equal(directory.enrichConversation({...conversation,previewSenderID:other}).previewSenderName,'max');
});

test('every inbox view sorts by recency, even when an older item has a higher importance score',()=>{
 const book=memoryBook();
 for(const id of ['new','old','invalid'])book.data.conversations[id]={priority:'important'};
 const items=[{id:'old',timestamp:'2026-10-01T00:00:00Z'},{id:'new',timestamp:'2026-11-09T00:00:00Z',attention:{important:true,reason:'Selected channel'}},{id:'invalid',timestamp:'invalid',attention:{important:true,reason:'Selected channel'}}];
 for(const view of ['all','focus'])assert.deepEqual(rankInbox(items,book,{view}).items.map(c=>c.id),['new','old','invalid']);
});
test('older bridge activity cannot push a newly active conversation back into October',()=>{
 const book=memoryBook(),directory=new Directory(book,{blue:async()=>[],config:()=>({})});
 directory.catalog.rooms.set('!room',{activity:{lastMessageAt:Date.parse('2026-10-01T00:00:00Z')}});
 const c=directory.enrichConversation({id:'matrix:!room',remoteId:'!room',participants:[],timestamp:'2026-11-09T00:00:00Z'});
 assert.equal(c.timestamp,'2026-11-09T00:00:00.000Z');
});
test('group photos use at most three distinct humans and exclude phone conversations',()=>{
 const book=memoryBook(),directory=new Directory(book,{blue:async()=>[],config:()=>({})});
 const keys=['me','bot','one','two','three','four'].map(s=>'@slack_'+s+':chord.local');
 keys.forEach((key,i)=>book.observe({key,name:['Me','Bridge','One','Two','Three','Four'][i],source:'slack',isBot:i===1}));
 directory.catalog.selfIds=new Set([keys[0]]);
 directory.catalog.rooms.set('!group',{direct:false,avatarMembers:keys});
 for(const source of ['slack','discord','whatsapp']){
  const group=directory.enrichConversation({id:'matrix:!group',remoteId:'!group',source,participants:[keys[2],keys[2]]});
  assert.deepEqual(group.avatarContacts.map(p=>p.name),['One','Two','Three']);
 }
 const phone=directory.enrichConversation({id:'phone',kind:'phone',source:'iphone',participants:keys,blue:{is_group:true}});
 assert.deepEqual(phone.avatarContacts,[]);
});

test('pins do not affect Important membership and Important does not affect badges',async()=>{
 const book=memoryBook();
 const keys=['pinned','important','both','neither'];
 const chats=keys.map(id=>({id,unread:true,unreadCount:3}));
 for(const id of ['important','both'])await book.setPreference(id,{priority:'important'});
 await book.setPreference('both',{muteMinutes:60});
 const view=rankInbox(chats,book,{view:'focus',pins:['pinned','both']});
 assert.deepEqual(view.items.map(chat=>chat.id),['important','both']);
 assert.equal(view.items[0].unreadCount,3);
 assert.equal(view.items[1].unread,false);
 assert.equal(rankInbox(chats,book).items.length,4);
 const quiet={id:'important',unread:false,sourceUnread:true};
 assert.equal(rankInbox([quiet],book,{view:'focus'}).items[0].unread,false);
});

test('unimportant conversations can be revealed or restored with unread state intact',async()=>{
 const book=memoryBook();
 const chats=[{id:'matrix:!chat',unread:true,unreadCount:3},{id:'mail:home:thread',unread:false}];
 await book.setPreference(chats[0].id,{priority:'unimportant'});
 assert.deepEqual(rankInbox(chats,book,{view:'inbox'}).items.map(c=>c.id),[chats[1].id]);
 const shown=rankInbox(chats,book,{view:'all'}).items[0];
 assert.equal(shown.localPriority,'unimportant');
 assert.equal(shown.unread,true);
 assert.equal(shown.unreadCount,3);
 await book.setPreference(chats[0].id,{priority:'auto'});
 assert.equal(rankInbox(chats,book,{view:'inbox'}).items.length,2);
});
