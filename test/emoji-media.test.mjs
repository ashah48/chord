import test from 'node:test';
import assert from 'node:assert/strict';
import {downloadEmoji} from '../emoji-media.mjs';

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
