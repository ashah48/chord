import test from 'node:test';
import assert from 'node:assert/strict';
import {downloadDiscordEmoji,downloadEmoji,downloadSlackEmoji} from '../emoji-media.mjs';

const account={homeserver:'http://127.0.0.1:4808',accessToken:'local-test'};

test('custom emoji downloads are limited to a small homeserver image',async()=>{
 let called=0;
 const fetcher=async(url,options)=>{
  called++;
  assert.equal(url.pathname,'/_matrix/client/v1/media/download/chord.local/abc');
  assert.equal(options.headers.Authorization,'Bearer local-test');
  return new Response(Buffer.from([1,2,3]),{status:200,headers:{'content-type':'image/png; charset=binary'}});
 };
 const image=await downloadEmoji(account,'mxc://chord.local/abc',fetcher);
 assert.equal(called,1);
 assert.equal(image.mimeType,'image/png');
 assert.equal(image.size,3);
 assert.equal(Buffer.from(image.data,'base64url').toString('hex'),'010203');
});

test('custom emoji downloads reject non-images and unsafe ids before fetching',async()=>{
 const fetcher=async()=>{throw new Error('should not fetch')};
 await assert.rejects(downloadEmoji(account,'https://example.com/a.png',fetcher),/Invalid custom emoji/);
 await assert.rejects(downloadEmoji(account,'mxc://chord.local/../../secret',fetcher),/Invalid custom emoji/);
 const html=async()=>new Response('<html>',{status:200,headers:{'content-type':'text/html'}});
 await assert.rejects(downloadEmoji(account,'mxc://chord.local/abc',html),{status:415});
});

test('custom emoji stop reading when the streamed body crosses the byte limit',async()=>{
 let canceled=false;
 const body=new ReadableStream({
  pull(controller){controller.enqueue(new Uint8Array(1024*1024))},
  cancel(){canceled=true},
 });
 const fetcher=async()=>new Response(body,{headers:{'content-type':'image/png'}});
 await assert.rejects(downloadEmoji(account,'mxc://chord.local/abc',fetcher),{status:413});
 assert.equal(canceled,true);
});

test('Discord emoji thumbnails use only numeric IDs on the Discord CDN',async()=>{
 const image=await downloadDiscordEmoji('123456789012345678',async(url,options)=>{
  assert.equal(url.href,'https://cdn.discordapp.com/emojis/123456789012345678.png?size=64');
  assert.equal(options.headers,undefined);
  return new Response(Buffer.from([1,2,3]),{headers:{'content-type':'image/png'}});
 });
 assert.equal(image.size,3);
 await assert.rejects(downloadDiscordEmoji('../avatar',async()=>assert.fail('unexpected request')),/Invalid Discord emoji/);
});

test('Slack thumbnails come from the Slack CDN without credentials or redirects',async()=>{
 const image=await downloadSlackEmoji('https://emoji.slack-edge.com/team/emoji.png',async(url, options)=>{
  assert.equal(url.hostname,'emoji.slack-edge.com');
  assert.equal(options.redirect,'error');
  assert.equal(options.headers,undefined);
  return new Response(Buffer.from([1]),{headers:{'content-type':'image/png'}});
 });
 assert.equal(image.size,1);
 await assert.rejects(downloadSlackEmoji('https://emoji.slack-edge.com.example.org/image',async()=>assert.fail()),{status:404});
});
