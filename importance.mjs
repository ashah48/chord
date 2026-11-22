export function importance(_conversation,{preference={}}={}){
 const important=preference.priority==='important';
 return {important,score:important?1000:0,reasons:important?['Marked important by you']:[],rule:important?'important':'auto'};
}

export function applyLocalRules(items,book,now=Date.now()){
 return items.map(c=>withLocalRules(c,book,now));
}
function withLocalRules(c,book,now){
 const preference=book.preference(c.id,now);
 // A cached row can still say isMuted after a local mute expires. Keep a network mute, which has no muteUntil.
 const networkMuted=!!c.isMuted&&c.muteUntil==null;
 const next={...c,localPriority:preference.priority,muteUntil:preference.muted?preference.muteUntil:null,isMuted:preference.muted||networkMuted};
 if(preference.muted)return {...next,unread:false,unreadCount:0,highlightCount:0,sourceUnread:c.sourceUnread??!!c.unread};
 return next;
}
export function rankInbox(items,book,{view='all',now=Date.now()}={}){
 const ranked=items.map(c=>{
  const next=withLocalRules(c,book,now);
  return {...next,importance:importance(next,{preference:book.preference(c.id,now)})};
 });
 const focused=ranked.filter(c=>c.importance.important);
 const visible=view==='focus'?focused:ranked;
 const timestamp=c=>{const value=Date.parse(c.timestamp);return Number.isFinite(value)?value:0};
 visible.sort((a,b)=>timestamp(b)-timestamp(a));
 return {items:visible,summary:{total:ranked.length,important:focused.length,hidden:view==='focus'?ranked.length-focused.length:0},view};
}
