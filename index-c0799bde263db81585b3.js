!function(e){function n(n){for(var t,r,l=n[0],i=n[1],a=0,s=[];a<l.length;a++)r=l[a],Object.prototype.hasOwnProperty.call(o,r)&&o[r]&&s.push(o[r][0]),o[r]=0;for(t in i)Object.prototype.hasOwnProperty.call(i,t)&&(e[t]=i[t]);for(c&&c(n);s.length;)s.shift()()}var t={},o={1:0};function r(n){if(t[n])return t[n].exports;var o=t[n]={i:n,l:!1,exports:{}};return e[n].call(o.exports,o,o.exports,r),o.l=!0,o.exports}r.e=function(e){var n=[],t=o[e];if(0!==t)if(t)n.push(t[2]);else{var l=new Promise((function(n,r){t=o[e]=[n,r]}));n.push(t[2]=l);var i,a=document.createElement("script");a.charset="utf-8",a.timeout=120,r.nc&&a.setAttribute("nonce",r.nc),a.src=function(e){return r.p+""+({}[e]||e)+"-"+{0:"034151694adf91cc34b3",3:"b4c99bb7ecb9a7c8ce61",4:"f930fbdd8b1b2ad47c2a"}[e]+".js"}(e);var c=new Error;i=function(n){a.onerror=a.onload=null,clearTimeout(s);var t=o[e];if(0!==t){if(t){var r=n&&("load"===n.type?"missing":n.type),l=n&&n.target&&n.target.src;c.message="Loading chunk "+e+" failed.\n("+r+": "+l+")",c.name="ChunkLoadError",c.type=r,c.request=l,t[1](c)}o[e]=void 0}};var s=setTimeout((function(){i({type:"timeout",target:a})}),12e4);a.onerror=a.onload=i,document.head.appendChild(a)}return Promise.all(n)},r.m=e,r.c=t,r.d=function(e,n,t){r.o(e,n)||Object.defineProperty(e,n,{enumerable:!0,get:t})},r.r=function(e){"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},r.t=function(e,n){if(1&n&&(e=r(e)),8&n)return e;if(4&n&&"object"==typeof e&&e&&e.__esModule)return e;var t=Object.create(null);if(r.r(t),Object.defineProperty(t,"default",{enumerable:!0,value:e}),2&n&&"string"!=typeof e)for(var o in e)r.d(t,o,function(n){return e[n]}.bind(null,o));return t},r.n=function(e){var n=e&&e.__esModule?function(){return e.default}:function(){return e};return r.d(n,"a",n),n},r.o=function(e,n){return Object.prototype.hasOwnProperty.call(e,n)},r.p="",r.oe=function(e){throw console.error(e),e};var l=this.webpackJsonp=this.webpackJsonp||[],i=l.push.bind(l);l.push=n,l=l.slice();for(var a=0;a<l.length;a++)n(l[a]);var c=i;r(r.s=2)}([function(e,n,t){e.exports=function(){return new Worker(t.p+"ebb6a20d9edcd9877b2d.worker.js")}},,function(e,n,t){"use strict";t.r(n);const o="https://tzl3kczlk5.execute-api.us-east-1.amazonaws.com/default/explainer",r=()=>{},l=()=>fetch(o,{method:"POST"}),i=(e,n)=>fetch(o,{method:"POST",body:JSON.stringify({kind:e,...n})});function a(e,n){let t={},o=null;return l=>{const i="function"==typeof l?l({...n.get(),...t}):l;r("setState: ",i),Object.assign(t,i),null==o&&(o=window.requestAnimationFrame(()=>(()=>{const r=Object.assign({},n.get(),t),l=n.get();n.set(r),t={},o=null,e(l)})()))}}function c(e){let n={_sentinel:{}};return t=>{const o=Object.keys({...n,...t}).some(e=>n[e]!==t[e]);n=t,o&&e(t)}}function s(e,n){e.classList.add(n)}function u(e,n){e.classList.remove(n)}function d(e,n){e.textContent=n}function m(e,n){e.innerHTML=n}function p(e,n){e.style.display=n}function h(e,n){let t=new window.URL(e),o=new window.URLSearchParams;return Object.entries(n).forEach(([e,n])=>{o.append(e,n)}),t.search=`?${o.toString()}`,t.toString()}self.addEventListener("error",e=>{i(null!=typeof window?"window.onerror":"self.onerror",{line:e&&e.lineno,column:e&&e.colno,message:e&&e.message,filename:e&&e.filename,stack:e&&e.error&&e.error.stack,raw:e})});var f=t(0),g=t.n(f);const b=e=>document.querySelector(e);const w={__sentinel:!0};function k(e){try{return localStorage.getItem(e)}catch(e){return y(e),w}}function v(e,n){try{localStorage.setItem(e,n)}catch(e){y(e)}}function y(e){i("storage",{raw:e,message:e.message,stack:e.stack})}const x=e=>document.querySelector(e);const M=({onNavigate:e,onOutsideClick:n})=>{const t=x(".missing-tooltip"),o=t.querySelector("code"),r=t.querySelector(".submit-issue");let l={missing:null};const i=a(()=>{!function({missing:e}){var n;(null!=e?s:u)(t,"visible"),d(o,null!==(n=null==e?void 0:e.code)&&void 0!==n?n:"")}({missing:l.missing})},{get:()=>l,set(e){l=e}});return r.addEventListener("click",n=>{n.preventDefault();const{code:t,location:o}=l.missing||{};if(!t||!o)return;const r=h("https://github.com/jrvidal/explaine.rs/issues/new",{labels:"missing-hint",title:"Missing Hint",body:["### What I expected","\x3c!-- What hint should we show here? What part of this syntax don't you understand? --\x3e","","### Source code","```",t,"```","",`Location: line ${o.line}, column ${o.ch}`].join("\n")});window.open(r,"_blank"),e()}),window.addEventListener("click",e=>{if(null==l.missing)return;let o=e.target;do{if(o==t)return;o=o.parentElement}while(null!=o);n()}),i};function S(){const e=x(".explanation"),n=e.querySelector(".loading"),t=e.querySelector(".loaded"),o=e.querySelector(".item-container"),r=o.querySelector(".item-title"),l=o.querySelector(".item"),i=o.querySelector(".error-message-container"),c=o.querySelector(".error-message"),h=e.querySelector(".file-bug"),f=e.querySelector(".do-file-bug"),g=l.innerHTML,b=r.innerHTML;let w={issueVisible:!1,error:null,compilationState:0,elaboration:null,missing:null};let y=null!=k("settings.reportDialog");const S=a(()=>{!function({error:e,compilationState:o,elaboration:a,missing:k}){p(n,0===o?"initial":"none"),p(t,0!==o?"initial":"none");const v=1===o&&null!=k;p(h,v?"block":"none"),(v&&!y?s:u)(f,"shake"),2===o?(m(r,"Oops! 💥"),m(l,"There is a syntax error in your code:"),p(i,"block"),d(c,e.msg)):null!=a?(m(r,a.title),m(l,a.elaboration),p(i,"none")):(m(r,b),m(l,g),p(i,"none"),E({missing:w.issueVisible?w.missing:null}))}(w)},{get:()=>w,set(e){w=e}}),E=M({onNavigate(){S({issueVisible:!1})},onOutsideClick(){S({issueVisible:!1})}});return f.addEventListener("click",e=>{e.preventDefault(),v("settings.reportDialog","true"),y=!0,S({issueVisible:!w.issueVisible})}),e=>S(e)}let E=Promise.all([t.e(0).then(t.t.bind(null,3,7)),Promise.all([t.e(0),t.e(4)]).then(t.t.bind(null,4,7)),Promise.all([t.e(0),t.e(3)]).then(t.t.bind(null,5,7))]);const L=window.document,I=e=>L.querySelector(e);l();const q="ontouchstart"in window;let O,T;s(L.body,q?"touch-device":"non-touch-device"),function({anchor:e,isTouchDevice:n,onChange:t,onMouseMove:o,onClick:r}){let l=E.then(([{default:l}])=>{const i=l.fromTextArea(e,{mode:"rust",lineNumbers:!0,theme:"solarized",readOnly:!!n&&"nocursor",indentUnit:4}),a=i.getWrapperElement();return i.on("change",e=>t(i,e)),a.addEventListener("mousemove",e=>o(i,e)),a.addEventListener("click",e=>r(i,e)),{cm:i,codemirrorEl:a}});return l.catch(e=>i("cmPromise",{message:e&&e.message,error:e})),l}({isTouchDevice:q,anchor:I(".codemirror-anchor"),onClick(){!function(e){1!==A.compilation.state||A.empty||(N.elaborationIndex=N.compilationIndex,N.lastElaborationRequest=e,W({type:"elaborate",location:e}))}(O.getCursor("from"))},onMouseMove(e,n){!function(e){if(1!==A.compilation.state)return;if(N.computedMarks)return void $(({compilation:n})=>({compilation:{...n,hoverEl:e.target}}));U(e)}(n)},onChange(){N.compilationIndex+=1,$({compilation:_,address:null,empty:""===O.getValue().trim()}),H()}}).then(({cm:e,codemirrorEl:n})=>(O=e,T=n,function(e){let n=Promise.resolve();const t=[...new window.URLSearchParams(location.search)].find(([e,n])=>"code"===e),o=null!=t?window.decodeURIComponent(t[1]):null;if(null!=o&&""!==o.trim())return e.setValue(o),n;const r=k("code");if("string"==typeof r&&""!==r.trim())return e.setValue(r),n;return n="loading"===L.readyState?new Promise(e=>{L.addEventListener("DOMContentLoaded",()=>e())}):Promise.resolve(),n=n.then(()=>e.setValue(I(".default-code").value)),n}(O))).then(()=>{s(L.body,"codemirror-rendered"),$({editorReady:!0})});const P=function({onAddress:e,getValue:n}){const t=b(".generate"),o=b(".link");return t.addEventListener("click",()=>{const t=n();null!=t&&e(h(window.location.href,{code:t}))}),c((function({address:e,enabled:n}){t.disabled=!n,e?(p(t,"none"),u(o,"hidden"),o.href=e):(p(t,"initial"),s(o,"hidden"))}))}({onAddress(e){$({address:e})},getValue:()=>O&&O.getValue()});!function(){const e=b(".modal"),n=b(".overlay");let t={showModal:!1};const o=a(o=>{t.showModal?(s(e,"show-modal"),s(n,"show-modal")):(u(e,"show-modal"),u(n,"show-modal"))},{get:()=>t,set(e){t=e}});b(".whats-this").addEventListener("click",()=>{o(({showModal:e})=>({showModal:!e}))}),n.addEventListener("click",()=>{o({showModal:!1})}),b(".close-modal").addEventListener("click",()=>{o({showModal:!1})})}();const C=function({onToggleEdit:e}){const n=b(".toggle-edit");return n.addEventListener("click",()=>{e()}),c(({enabled:e,editable:t})=>{n.disabled=!e,d(n,t?"Disable editing":"Enable editing")})}({onToggleEdit(){$(({editable:e})=>({editable:!e}))}}),V=function({onToggleShowAll:e}){const n=b(".show-all"),t=b(".show-all-text"),o=t.textContent;return n.addEventListener("click",()=>{e()}),c((function({showAll:e,enabled:r}){n.disabled=!r,(null!=e?s:u)(n,"show-all-loaded"),d(t,!0===e?"Hide elements":o)}))}({onToggleShowAll(){$(({compilation:e})=>({compilation:{...e,exploration:null!=e.exploration?{showAll:!e.exploration.showAll}:null}}))}}),R=function({getValue:e}){const n=b(".playground");return n.addEventListener("click",()=>{const n=e();null!=n&&window.open(h("https://play.rust-lang.org",{code:n,edition:"2018"}),"_blank")}),c((function({enabled:e}){n.disabled=!e}))}({getValue:()=>O&&O.getValue()}),j=function({onWrapInBlock:e}){const n=x(".explanation").querySelector(".more-info"),t=n.querySelector(".book-row"),o=t.querySelector("a"),r=n.querySelector(".keyword-row"),l=r.querySelector("a"),i=n.querySelector(".std-row"),a=i.querySelector("a"),s=x(".info-wip"),u=function({onWrapInBlock:e}){const n=x(".can-be-block");return x(".wrap-in-block").addEventListener("click",()=>{e()}),({canBeBlock:e})=>{p(n,e?"block":"none")}}({onWrapInBlock:e}),d=S(),m=c(u),h=c((function({elaboration:e}){null!=e?(p(n,"block"),p(t,e.book?"block":"none"),p(r,e.keyword?"block":"none"),p(i,e.std?"block":"none"),o.href=e.book||"",l.href=e.keyword||"",a.href=e.std||"",p(s,e.book||e.keyword||e.std?"none":"initial")):p(n,"none")})),f=c(d);return({elaboration:e,error:n,compilationState:t,missing:o})=>{m({canBeBlock:Boolean(n&&n.isBlock)}),h({elaboration:e}),f({error:n,compilationState:t,elaboration:e,missing:o})}}({onWrapInBlock(){if(null==O)return;const e=O.lineCount();for(let n=0;n<e;n++)O.indentLine(n,"add");O.replaceRange("fn main() {\n",{line:0,ch:0}),O.replaceRange("\n}",{line:e,ch:O.getLineHandle(e).text.length})}});let A={compilation:{state:0,hoverEl:null,explanation:null,elaboration:null,exploration:null,error:null,missing:null},editable:!q,address:null,editorReady:!1,empty:!1};const _=A.compilation;let N={lastRule:-1,mark:null,hoverMark:null,computedMarks:null,errorMark:null,errorContextMark:null,hoverIndex:null,compilationIndex:0,elaborationIndex:null,lastElaborationRequest:null};const $=a(()=>{var e,n,t;const{compilation:o}=A;X({hoverEl:o.hoverEl}),G({error:o.error}),K({elaboration:o.elaboration}),Q({explanation:o.explanation}),Z({showAll:null!==(n=null===(e=o.exploration)||void 0===e?void 0:e.showAll)&&void 0!==n&&n,editable:A.editable}),j({elaboration:o.elaboration,error:o.error,compilationState:o.state,missing:o.missing}),P({address:A.address,enabled:A.editorReady}),C({editable:A.editable,enabled:A.editorReady}),V({showAll:!A.empty&&2!==o.state&&(null===(t=o.exploration)||void 0===t?void 0:t.showAll),enabled:null!=o.exploration}),R({enabled:!A.empty&&1===o.state})},{get:()=>A,set(e){A=e}});let{postMessage:W,ready:B}=function({onMessage:e}){const n=new g.a;let t,o=new Promise(e=>{t=e});return n.onerror=e=>i("worker.onerror",e),n.onmessageerror=e=>i("onmessageerror",{error:e}),n.onmessage=n=>{const{data:o}=n;r("Window received",o.type,o),"ready"!==o.type?e(o):t()},{postMessage:e=>n.postMessage(e),ready:o}}({onMessage(e){switch(e.type){case"compiled":$({compilation:{..._,state:1}});break;case"compilation-error":$({compilation:{..._,state:2,error:e.error}});break;case"explanation":$(({compilation:n})=>({compilation:{...n,explanation:e.location}})),function(){if(N.computedMarks)return;z()}();break;case"elaboration":!function(e){if(N.compilationIndex!==N.elaborationIndex)return;const n=null==e.location?function({line:e,ch:n}){const t=/^ *$/;if(t.test(O.getLine(e)))return null;const o=Math.max(0,e-5),r=Math.min(O.lineCount()-1,e+5);let l=[...new Array(r-o+1)].map((e,n)=>O.getLine(o+n));const i=l.map(e=>{var n,o;return t.test(e)?Number.POSITIVE_INFINITY:null!==(o=null===(n=e.match(/^ */))||void 0===n?void 0:n[0].length)&&void 0!==o?o:0});let a=Math.min(...i);a=a===Number.POSITIVE_INFINITY?0:Math.min(a,n),a>0&&l.forEach((e,n)=>{t.test(e)||(l[n]=e.slice(a))});return l.forEach((e,n)=>{l[n]=`${String(n).padStart(2," ")} | ${e}`}),l.splice(e-o+1,0,`   | ${" ".repeat(n-a)}↑`),{code:l.join("\n"),location:{line:e-o,ch:n-a}}}(N.lastElaborationRequest):null;$(({compilation:t})=>({compilation:{...t,elaboration:null!=e.location?e:null,missing:n}}))}(e);break;case"exploration":!function(e){N.computedMarks=e.map(({start:e,end:n},t)=>ee({start:e,end:n},`computed-${t}`));for(let n=N.lastRule+1;n<e.length;n++)D.insertRule(`.hover-${n} .computed-${n} { background: #eee8d5 }`,D.cssRules.length);N.lastRule=Math.max(e.length,N.lastRule),N.hoverMark&&N.hoverMark.clear()}(e.exploration),$(({compilation:e})=>({compilation:{...e,exploration:{showAll:!1}}}));break;default:console.error("Unexpected message in window",e)}}});const D=(()=>{const e=L.createElement("style");return L.head.appendChild(e),e.sheet})();const H=(()=>{let e=!1;B.then(()=>{e=!0});let n=!1;return()=>{e?Y():n||(n=!0,B.then(()=>Y()))}})();const{debounced:U,done:z}=function(e,n){let t=!0;const o={};let r=o,l=!1;const i=()=>{r!==o?l||(l=!0,window.setTimeout(()=>{l=!1;const n=r;r=o,e(n,i)},n)):t=!0};return{done:i,debounced(n){t?(t=!1,e(n,i)):r=n}}}((function({clientX:e,clientY:n},t){const{compilation:o}=A;if(1!==o.state)return t();let{line:r,ch:l}=O.coordsChar({left:e,top:n},"window");F({line:r,ch:l}),F.cached&&t()}),200),F=function(e,n){let t={},o=r=>{n(t,r)?(t=r,o.cached=!0):(t=r,o.cached=!1,e(r))};return o}((function({line:e,ch:n}){W({type:"explain",location:{line:e,ch:n}})}),(e,n)=>{if(e.line===n.line&&e.ch===n.ch)return!0;const{explanation:t}=A.compilation;return null!=t&&function({line:e,ch:n},t,o){return(t.line<e||t.line===e&&t.ch<=n)&&(e<o.line||e===o.line&&n<=o.ch)}(n,t.start,t.end)});const J=function(e,n){let t=null,o=null;const r=()=>e(o);return e=>{o=e,null!=t&&window.clearTimeout(t),t=window.setTimeout(r,n)}}(e=>W({type:"compile",source:e}),128);function Y(){const e=O.getValue();v("code",e),""===e.trim()?$({compilation:{..._,state:1}}):J(O.getValue());const{computedMarks:n}=N;N.computedMarks=null,n&&requestAnimationFrame(()=>n.forEach(e=>e.clear()))}const X=c((function({hoverEl:e}){const n=e&&[...e.classList].find(e=>e.startsWith("computed-")),t=null!=n?Number(n.replace("computed-","")):null;null!=N.hoverIndex&&t!==N.hoverIndex&&u(T,`hover-${N.hoverIndex}`),null!=t&&s(T,`hover-${t}`),N.hoverIndex=t})),G=c((function({error:e}){N.errorMark&&N.errorMark.clear(),N.errorContextMark&&N.errorContextMark.clear(),null!=e&&(N.errorMark=ee(e,"compilation-error"),N.errorContextMark=ee({start:{...e.start,ch:0},end:{...e.end,ch:O.getLine(e.end.line).length}},"compilation-error"))})),K=c((function({elaboration:e}){N.mark&&N.mark.clear(),null!=e&&(N.mark=ee(e.location))})),Q=c((function({explanation:e}){N.hoverMark&&N.hoverMark.clear(),null!=e&&null==N.computedMarks&&(N.hoverMark=ee(e))})),Z=c((function({showAll:e,editable:n}){O.setOption("readOnly",!n&&"nocursor"),e?s(T,"show-all-computed"):u(T,"show-all-computed")}));function ee(e,n="highlighted"){return O.markText(e.start,e.end,{className:n})}}]);