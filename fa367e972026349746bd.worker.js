!function(e){var t={};function n(r){if(t[r])return t[r].exports;var o=t[r]={i:r,l:!1,exports:{}};return e[r].call(o.exports,o,o.exports,n),o.l=!0,o.exports}n.m=e,n.c=t,n.d=function(e,t,r){n.o(e,t)||Object.defineProperty(e,t,{enumerable:!0,get:r})},n.r=function(e){"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},n.t=function(e,t){if(1&t&&(e=n(e)),8&t)return e;if(4&t&&"object"==typeof e&&e&&e.__esModule)return e;var r=Object.create(null);if(n.r(r),Object.defineProperty(r,"default",{enumerable:!0,value:e}),2&t&&"string"!=typeof e)for(var o in e)n.d(r,o,function(t){return e[t]}.bind(null,o));return r},n.n=function(e){var t=e&&e.__esModule?function(){return e.default}:function(){return e};return n.d(t,"a",t),t},n.o=function(e,t){return Object.prototype.hasOwnProperty.call(e,t)},n.p="",n(n.s=1)}([function(e,t){let n;!function(){const e={};let t,r=new TextDecoder("utf-8",{ignoreBOM:!0,fatal:!0});r.decode();let o=null;function s(){return null!==o&&o.buffer===t.memory.buffer||(o=new Uint8Array(t.memory.buffer)),o}function i(e,t){return r.decode(s().subarray(e,e+t))}const a=new Array(32).fill(void 0);a.push(void 0,null,!0,!1);let l=a.length;let u=null;function c(){return null!==u&&u.buffer===t.memory.buffer||(u=new Int32Array(t.memory.buffer)),u}let p=null;function _(){return null!==p&&p.buffer===t.memory.buffer||(p=new Uint32Array(t.memory.buffer)),p}function f(e){const t=function(e){return a[e]}(e);return function(e){e<36||(a[e]=l,l=e)}(e),t}let b=0,d=new TextEncoder("utf-8");const g="function"==typeof d.encodeInto?function(e,t){return d.encodeInto(e,t)}:function(e,t){const n=d.encode(e);return t.set(n),{read:e.length,written:n.length}};class m{static __wrap(e){const t=Object.create(m.prototype);return t.ptr=e,t}free(){const e=this.ptr;this.ptr=0,t.__wbg_explanation_free(e)}get start_line(){return t.__wbg_get_explanation_start_line(this.ptr)>>>0}set start_line(e){t.__wbg_set_explanation_start_line(this.ptr,e)}get start_column(){return t.__wbg_get_explanation_start_column(this.ptr)>>>0}set start_column(e){t.__wbg_set_explanation_start_column(this.ptr,e)}get end_line(){return t.__wbg_get_explanation_end_line(this.ptr)>>>0}set end_line(e){t.__wbg_set_explanation_end_line(this.ptr,e)}get end_column(){return t.__wbg_get_explanation_end_column(this.ptr)>>>0}set end_column(e){t.__wbg_set_explanation_end_column(this.ptr,e)}elaborate(){return f(t.explanation_elaborate(this.ptr))}title(){return f(t.explanation_title(this.ptr))}keyword(){return f(t.explanation_keyword(this.ptr))}book(){return f(t.explanation_book(this.ptr))}std(){return f(t.explanation_std(this.ptr))}}e.Explanation=m;class w{static __wrap(e){const t=Object.create(w.prototype);return t.ptr=e,t}free(){const e=this.ptr;this.ptr=0,t.__wbg_session_free(e)}static new(e){var n=function(e,t,n){if(void 0===n){const n=d.encode(e),r=t(n.length);return s().subarray(r,r+n.length).set(n),b=n.length,r}let r=e.length,o=t(r);const i=s();let a=0;for(;a<r;a++){const t=e.charCodeAt(a);if(t>127)break;i[o+a]=t}if(a!==r){0!==a&&(e=e.slice(a)),o=n(o,r,r=a+3*e.length);const t=s().subarray(o+a,o+r);a+=g(e,t).written}return b=a,o}(e,t.__wbindgen_malloc,t.__wbindgen_realloc),r=b,o=t.session_new(n,r);return y.__wrap(o)}explore(e){try{var n=function(e,t){const n=t(4*e.length);return _().set(e,n/4),b=e.length,n}(e,t.__wbindgen_malloc),r=b;return t.session_explore(this.ptr,n,r)>>>0}finally{e.set(_().subarray(n/4,n/4+r)),t.__wbindgen_free(n,4*r)}}explain(e,n){var r=t.session_explain(this.ptr,e,n);return 0===r?void 0:m.__wrap(r)}}e.Session=w;class y{static __wrap(e){const t=Object.create(y.prototype);return t.ptr=e,t}free(){const e=this.ptr;this.ptr=0,t.__wbg_sessionresult_free(e)}session(){var e=this.ptr;this.ptr=0;var n=t.sessionresult_session(e);return 0===n?void 0:w.__wrap(n)}error_location(){t.sessionresult_error_location(8,this.ptr);var e=c()[2],n=c()[3];let r;var o,s;return 0!==e&&(r=(o=e,s=n,_().subarray(o/4,o/4+s)).slice(),t.__wbindgen_free(e,4*n)),r}error_message(){return f(t.sessionresult_error_message(this.ptr))}is_block(){return 0!==t.sessionresult_is_block(this.ptr)}}e.SessionResult=y,n=Object.assign((async function e(n){if(void 0===n){let e;e="undefined"==typeof document?location.href:document.currentScript.src,n=e.replace(/\.js$/,"_bg.wasm")}const r={wbg:{}};r.wbg.__wbindgen_string_new=function(e,t){return function(e){l===a.length&&a.push(a.length+1);const t=l;return l=a[t],a[t]=e,t}(i(e,t))},r.wbg.__wbindgen_throw=function(e,t){throw new Error(i(e,t))},("string"==typeof n||"function"==typeof Request&&n instanceof Request||"function"==typeof URL&&n instanceof URL)&&(n=fetch(n));const{instance:o,module:s}=await async function(e,t){if("function"==typeof Response&&e instanceof Response){if("function"==typeof WebAssembly.instantiateStreaming)try{return await WebAssembly.instantiateStreaming(e,t)}catch(t){if("application/wasm"==e.headers.get("Content-Type"))throw t;console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n",t)}const n=await e.arrayBuffer();return await WebAssembly.instantiate(n,t)}{const n=await WebAssembly.instantiate(e,t);return n instanceof WebAssembly.Instance?{instance:n,module:e}:n}}(await n,r);return t=o.exports,e.__wbindgen_wasm_module=s,t}),e)}(),e.exports=n},function(e,t,n){"use strict";n.r(t);var r=n(0),o=n.n(r);const s="https://tzl3kczlk5.execute-api.us-east-1.amazonaws.com/default/explainer",i=()=>{},a=e=>fetch(s,{method:"POST",body:JSON.stringify(e)});self.addEventListener("error",e=>{a({line:e&&e.lineno,column:e&&e.colno,message:e&&e.message,filename:e&&e.filename,stack:e&&e.error&&e.error.stack,raw:e})});var l=n.p+"b4a0469285d4561da33e6776b9c152d7.wasm";i("workerMain");const u={source:null,session:null,explanation:null,exploration:null};function c(e){u.explanation&&(u.explanation.free(),u.explanation=null),u.explanation=u.session&&u.session.explain(e.line+1,e.ch)}function p(e){return null!=e?{start:{line:e.start_line-1,ch:e.start_column},end:{line:e.end_line-1,ch:e.end_column}}:null}o()(l).then(()=>postMessage({type:"ready"})).catch(e=>a({error:e,message:e&&e.message})),self.onmessage=e=>{const{data:t}=e;switch(i("Worker received",t.type,t),t.type){case"compile":!function(e){u.session&&(u.session.free(),u.session=null);const t=o.a.Session.new(e),n=t.error_message(),r=t.error_location(),s=null!=n?{msg:n,start:{line:r[0]-1,ch:r[1]},end:{line:r[2]-1,ch:r[3]},isBlock:t.is_block()}:null;u.session=t.session(),u.error=s,postMessage({type:null!=u.session?"compiled":"compilation-error",error:u.error}),function e(t,n=!1){if(t!=u.session||null==u.session)return;n&&(u.exploration={buffer:new self.Uint32Array(20),result:[]});const{buffer:r}=u.exploration,o=(Date.now(),u.session.explore(r));for(let e=0;e<o;e++)u.exploration.result.push([[r[4*e]-1,r[4*e+1]],[r[4*e+2]-1,r[4*e+3]]]);if(o<5)return i("Exploration finished..."),void postMessage({type:"exploration",exploration:u.exploration.result.map(e=>{const[t,n]=e;return{start:{line:t[0],ch:t[1]},end:{line:n[0],ch:n[1]}}})});setTimeout(()=>e(t),16)}(u.session,!0)}(t.source);break;case"explain":!function(e){c(e),postMessage({type:"explanation",location:p(u.explanation)})}(t.location);break;case"elaborate":!function(e){c(e),postMessage({type:"elaboration",location:p(u.explanation),elaboration:u.explanation&&u.explanation.elaborate(),title:u.explanation&&u.explanation.title(),book:u.explanation&&u.explanation.book(),keyword:u.explanation&&u.explanation.keyword(),std:u.explanation&&u.explanation.std()})}(t.location);break;default:console.error("Unexpected message in worker",t)}}}]);