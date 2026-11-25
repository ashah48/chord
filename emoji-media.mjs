// Discord and Slack custom reactions are bridged as mxc images. Download only
// that image from the signed-in homeserver, and refuse anything that is not a
// small picture.
const MXC=/^mxc:\/\/([A-Za-z0-9._:-]+)\/([A-Za-z0-9_-]+)$/;
const MAX_BYTES=2*1024*1024;

export async function downloadEmoji(account,mxc,fetcher=fetch){
 const match=MXC.exec(String(mxc||''));
 if(!match||!account?.homeserver||!account?.accessToken)throw Object.assign(new Error('Invalid custom emoji'),{status:400});
 const url=new URL(`/_matrix/client/v1/media/download/${encodeURIComponent(match[1])}/${encodeURIComponent(match[2])}`,account.homeserver);
 const response=await fetcher(url,{headers:{Authorization:'Bearer '+account.accessToken},signal:AbortSignal.timeout(15000)});
 return emojiResponse(response);
}

export async function downloadDiscordEmoji(id,fetcher=fetch){
 if(!/^\d{15,22}$/.test(id))throw Object.assign(new Error('Invalid Discord emoji'),{status:400});
 const response=await fetcher(new URL(`https://cdn.discordapp.com/emojis/${id}.png?size=64`),{signal:AbortSignal.timeout(10000)});
 return emojiResponse(response);
}

export async function downloadSlackEmoji(address,fetcher=fetch){
 const url=URL.parse(address||'');
 if(url?.protocol!=='https:'||!['emoji.slack-edge.com','a.slack-edge.com'].includes(url.hostname)
    ||url.port||url.username||url.password){
  throw Object.assign(new Error('Slack emoji image unavailable'),{status:404});
 }
 return emojiResponse(await fetcher(url,{redirect:'error',signal:AbortSignal.timeout(10000)}));
}

async function emojiResponse(response){
 if(!response.ok)throw Object.assign(new Error('Custom emoji unavailable'),{status:response.status||502});
 const mimeType=(response.headers.get('content-type')||'').split(';')[0].trim().toLowerCase();
 if(!mimeType.startsWith('image/'))throw Object.assign(new Error('Custom emoji is not an image'),{status:415});
 const rejectSize=()=>Object.assign(new Error('Custom emoji is empty or exceeds 2 MB'),{status:413});
 if(!response.body)throw rejectSize();
 const reader=response.body.getReader(),chunks=[];
 let size=0;
 try{
  while(true){
   const {value,done}=await reader.read();if(done)break;
   size+=value.byteLength;
   if(size>MAX_BYTES)throw rejectSize();
   chunks.push(value);
  }
 }finally{await reader.cancel().catch(()=>{})}
 if(!size)throw rejectSize();
 const bytes=Buffer.concat(chunks,size);
 return {data:bytes.toString('base64url'),mimeType,size:bytes.length};
}
