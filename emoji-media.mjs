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
 if(!response.ok)throw Object.assign(new Error('Custom emoji unavailable'),{status:response.status||502});
 const mimeType=(response.headers.get('content-type')||'').split(';')[0].trim().toLowerCase();
 if(!mimeType.startsWith('image/'))throw Object.assign(new Error('Custom emoji is not an image'),{status:415});
 const bytes=Buffer.from(await response.arrayBuffer());
 if(!bytes.length||bytes.length>MAX_BYTES)throw Object.assign(new Error('Custom emoji is too large'),{status:413});
 return {data:bytes.toString('base64url'),mimeType,size:bytes.length};
}
