!function(e){function n(n){for(var o,r,a=n[0],i=n[1],l=0,s=[];l<a.length;l++)r=a[l],Object.prototype.hasOwnProperty.call(t,r)&&t[r]&&s.push(t[r][0]),t[r]=0;for(o in i)Object.prototype.hasOwnProperty.call(i,o)&&(e[o]=i[o]);for(c&&c(n);s.length;)s.shift()()}var o={},t={1:0};function r(n){if(o[n])return o[n].exports;var t=o[n]={i:n,l:!1,exports:{}};return e[n].call(t.exports,t,t.exports,r),t.l=!0,t.exports}r.e=function(e){var n=[],o=t[e];if(0!==o)if(o)n.push(o[2]);else{var a=new Promise((function(n,r){o=t[e]=[n,r]}));n.push(o[2]=a);var i,l=document.createElement("script");l.charset="utf-8",l.timeout=120,r.nc&&l.setAttribute("nonce",r.nc),l.src=function(e){return r.p+""+({}[e]||e)+"-"+{0:"62742bf830e0ca430378",3:"d39f93f7895244bf161c",4:"a49075af1a9f2c11d246"}[e]+".js"}(e);var c=new Error;i=function(n){l.onerror=l.onload=null,clearTimeout(s);var o=t[e];if(0!==o){if(o){var r=n&&("load"===n.type?"missing":n.type),a=n&&n.target&&n.target.src;c.message="Loading chunk "+e+" failed.\n("+r+": "+a+")",c.name="ChunkLoadError",c.type=r,c.request=a,o[1](c)}t[e]=void 0}};var s=setTimeout((function(){i({type:"timeout",target:l})}),12e4);l.onerror=l.onload=i,document.head.appendChild(l)}return Promise.all(n)},r.m=e,r.c=o,r.d=function(e,n,o){r.o(e,n)||Object.defineProperty(e,n,{enumerable:!0,get:o})},r.r=function(e){"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},r.t=function(e,n){if(1&n&&(e=r(e)),8&n)return e;if(4&n&&"object"==typeof e&&e&&e.__esModule)return e;var o=Object.create(null);if(r.r(o),Object.defineProperty(o,"default",{enumerable:!0,value:e}),2&n&&"string"!=typeof e)for(var t in e)r.d(o,t,function(n){return e[n]}.bind(null,t));return o},r.n=function(e){var n=e&&e.__esModule?function(){return e.default}:function(){return e};return r.d(n,"a",n),n},r.o=function(e,n){return Object.prototype.hasOwnProperty.call(e,n)},r.p="",r.oe=function(e){throw console.error(e),e};var a=window.webpackJsonp=window.webpackJsonp||[],i=a.push.bind(a);a.push=n,a=a.slice();for(var l=0;l<a.length;l++)n(a[l]);var c=i;r(r.s=0)}([function(e,n,o){const t="undefined"!=typeof WorkerGlobalScope&&self instanceof WorkerGlobalScope;(t?async function(){console.info("workerMain");const e={source:null,session:null,explanation:null};function n(n){e.session&&(e.session.free(),e.session=null),e.session=wasm_bindgen.Session.new(n),postMessage({type:null!=e.session?"compiled":"compilation-error"})}function o(n){r(n),postMessage({type:"explanation",location:a(e.explanation)})}function t(n){r(n),postMessage({type:"elaboration",location:a(e.explanation),elaboration:e.explanation&&e.explanation.elaborate(),title:e.explanation&&e.explanation.title()})}function r(n){e.explanation&&(e.explanation.free(),e.explanation=null),e.explanation=e.session&&e.session.explain(n.line+1,n.ch)}function a(e){return null!=e?{start:{line:e.start_line-1,ch:e.start_column},end:{line:e.end_line-1,ch:e.end_column}}:null}self.onmessage=e=>{const{data:r}=e;switch(console.info("Worker received",r.type),r.type){case"load":!async function(e){self.importScripts(e);try{await wasm_bindgen(e.replace(/\.js$/,"_bg.wasm"))}catch(e){return void console.error(e)}postMessage({type:"ready"})}(r.url);break;case"compile":n(r.source);break;case"explain":o(r.location);break;case"elaborate":t(r.location);break;default:console.error("Unexpected message in worker",r)}}}():async function(){await o.e(0).then(o.t.bind(null,2,7)),await Promise.all([o.e(0),o.e(4)]).then(o.t.bind(null,3,7)),await Promise.all([o.e(0),o.e(3)]).then(o.t.bind(null,4,7));let e={compilation:null};const n=new Worker(window.workerMain,{name:"explainer"});n.onerror=e=>console.error(e),n.onmessage=n=>{const{data:o}=n;switch(console.info("Window received",o.type),console.info("Window state:",e),o.type){case"ready":e.compilation={state:"pending"},u();break;case"compiled":e.compilation={state:"success"},c();break;case"compilation-error":e.compilation={state:"error"},c();break;case"explanation":e.compilation&&(e.compilation.explanation=o.location,e.compilation.computing=!1,function(){const n=e.compilation&&e.compilation.explanation;if(e.compilation.hoverMark&&e.compilation.hoverMark.clear(),null==n)return;e.compilation.hoverMark=p(n)}());break;case"elaboration":e.compilation&&(e.compilation.elaboration=null!=o.location?{location:o.location,elaboration:o.elaboration,title:o.title}:null,function(){const{compilation:n}=e;if(n.mark&&n.mark.clear(),null==(n&&n.elaboration))return;n.hoverMark&&n.hoverMark.clear(),n.mark=p(n.elaboration.location),r.innerHTML=n.elaboration.title,a.innerHTML=n.elaboration.elaboration}());break;default:console.error("Unexpected message in window",o)}console.info("Window state after:",e)},n.postMessage({type:"load",url:window.workerBundle});const t=document.querySelector("#explanation").querySelector(".item-container"),r=t.querySelector(".item-title"),a=t.querySelector(".item"),i=document.querySelector(".error"),l=CodeMirror.fromTextArea(document.querySelector("#editor"),{mode:"rust",lineNumbers:!0,theme:"solarized"});function c(){const{compilation:n}=e;var o;s()?(i.style.display="inherit",o=()=>{s()&&(i.style.opacity=1)},requestAnimationFrame(()=>requestAnimationFrame(()=>o()))):(i.style.display="none",i.style.opacity="0")}function s(){return null!=e.compilation&&"error"===e.compilation.state}function u(){""!==l.getValue().trim()&&n.postMessage({type:"compile",source:l.getValue()})}function p(e){return l.markText(e.start,e.end,{className:"highlighted"})}l.on("change",()=>{u()}),l.getWrapperElement().addEventListener("mousemove",o=>{const{compilation:t}=e;if(null==t||"success"!==t.state||t.computing)return;t.computing=!0;const{clientX:r,clientY:a}=o;!function(o,t){const{compilation:r}=e;if(null==r||"success"!==r.state)return;let{line:a,ch:i}=l.coordsChar({left:o,top:t},"window");const c=r.lastLocation||{};a!==c.line||i!==c.ch?(r.lastLocation={line:a,ch:i},n.postMessage({type:"explain",location:{line:a,ch:i}})):r.computing=!1}(r,a)}),l.getWrapperElement().addEventListener("click",()=>{var o;o=l.getCursor("from"),"success"===(e.compilation&&e.compilation.state)&&n.postMessage({type:"elaborate",location:o})}),window.cm=l}()).catch(e=>console.error(e))}]);