"use strict";(self.webpackChunkhips_webgl_renderer=self.webpackChunkhips_webgl_renderer||[]).push([[995],{1995:(e,t,r)=>{r.r(t),r.d(t,{BMOC:()=>n.Qi,Cell:()=>n.bL,Neighbours:()=>n.X,PolygonMap:()=>n.xC,__wbindgen_throw:()=>n.Or,getOrderMax:()=>n.Zu,lonlatToNested:()=>n.yv,multiLonlatToNested:()=>n.fx,nestedCenter:()=>n.xY,nestedNeighbours:()=>n.zi,nestedQueryCone:()=>n.RH,nestedQueryConeBMOC:()=>n.NT,nestedQueryPolygon:()=>n.LH,nestedQueryPolygonBMOC:()=>n.K1,nestedVertices:()=>n.Qo});var n=r(821)},821:(e,t,r)=>{r.d(t,{Zu:()=>u,yv:()=>h,fx:()=>w,xY:()=>d,Qo:()=>y,zi:()=>m,NT:()=>v,RH:()=>C,K1:()=>x,LH:()=>O,Qi:()=>Q,bL:()=>M,X:()=>N,xC:()=>P,Or:()=>T});var n=r(8531);e=r.hmd(e);var _=r(8731).TextDecoder;let o=new(void 0===_?(0,e.require)("util").TextDecoder:_)("utf-8",{ignoreBOM:!0,fatal:!0});o.decode();let s=null;function u(){return n.getOrderMax()}let l=null;function i(){return null!==l&&l.buffer===n.memory.buffer||(l=new Float64Array(n.memory.buffer)),l}let g=0;function a(e,t){const r=t(8*e.length);return i().set(e,r/8),g=e.length,r}let c=null;function b(){return null!==c&&c.buffer===n.memory.buffer||(c=new Int32Array(n.memory.buffer)),c}let p=null;function h(e,t,r){return n.lonlatToNested(e,t,r)}function f(e,t){return i().subarray(e/8,e/8+t)}function w(e,t){var r=a(t,n.__wbindgen_malloc),_=g;n.multiLonlatToNested(8,e,r,_);var o=b()[2],s=b()[3],u=f(o,s).slice();return n.__wbindgen_free(o,8*s),u}function d(e,t){n.nestedCenter(8,e,t);var r=b()[2],_=b()[3],o=f(r,_).slice();return n.__wbindgen_free(r,8*_),o}function y(e,t){n.nestedVertices(8,e,t);var r=b()[2],_=b()[3],o=f(r,_).slice();return n.__wbindgen_free(r,8*_),o}function m(e,t){var r=n.nestedNeighbours(e,t);return N.__wrap(r)}function v(e,t,r,_){var o=n.nestedQueryConeBMOC(e,t,r,_);return Q.__wrap(o)}function C(e,t,r,_){n.nestedQueryCone(8,e,t,r,_);var o=b()[2],s=b()[3],u=f(o,s).slice();return n.__wbindgen_free(o,8*s),u}function x(e,t){var r=a(t,n.__wbindgen_malloc),_=g,o=n.nestedQueryPolygonBMOC(e,r,_);return Q.__wrap(o)}function O(e,t){var r=a(t,n.__wbindgen_malloc),_=g;n.nestedQueryPolygon(8,e,r,_);var o=b()[2],s=b()[3],u=f(o,s).slice();return n.__wbindgen_free(o,8*s),u}class Q{static __wrap(e){const t=Object.create(Q.prototype);return t.ptr=e,t}free(){const e=this.ptr;this.ptr=0,n.__wbg_bmoc_free(e)}get n_cells(){return n.__wbg_get_bmoc_n_cells(this.ptr)>>>0}get depth_max(){return n.__wbg_get_bmoc_depth_max(this.ptr)}getSize(){return n.__wbg_get_bmoc_n_cells(this.ptr)>>>0}getDepth(){return n.__wbg_get_bmoc_depth_max(this.ptr)}getCell(e){var t=n.bmoc_getCell(this.ptr,e);return M.__wrap(t)}getCellDepth(e){return n.bmoc_getCellDepth(this.ptr,e)}getCellHash(e){return n.bmoc_getCellHash(this.ptr,e)}getCellFlag(e){return 0!==n.bmoc_getCellFlag(this.ptr,e)}}class M{static __wrap(e){const t=Object.create(M.prototype);return t.ptr=e,t}free(){const e=this.ptr;this.ptr=0,n.__wbg_cell_free(e)}get order(){return n.__wbg_get_cell_order(this.ptr)}get icell(){return n.__wbg_get_cell_icell(this.ptr)}get isfull(){return 0!==n.__wbg_get_cell_isfull(this.ptr)}}class N{static __wrap(e){const t=Object.create(N.prototype);return t.ptr=e,t}free(){const e=this.ptr;this.ptr=0,n.__wbg_neighbours_free(e)}get south(){n.__wbg_get_neighbours_south(8,this.ptr);var e=b()[2],t=i()[2];return 0===e?void 0:t}get southeast(){return n.__wbg_get_neighbours_southeast(this.ptr)}get east(){n.__wbg_get_neighbours_east(8,this.ptr);var e=b()[2],t=i()[2];return 0===e?void 0:t}get southwest(){return n.__wbg_get_neighbours_southwest(this.ptr)}get center(){return n.__wbg_get_neighbours_center(this.ptr)}get norhteast(){return n.__wbg_get_neighbours_norhteast(this.ptr)}get west(){n.__wbg_get_neighbours_west(8,this.ptr);var e=b()[2],t=i()[2];return 0===e?void 0:t}get norhtwest(){return n.__wbg_get_neighbours_norhtwest(this.ptr)}get north(){n.__wbg_get_neighbours_north(8,this.ptr);var e=b()[2],t=i()[2];return 0===e?void 0:t}}class P{static __wrap(e){const t=Object.create(P.prototype);return t.ptr=e,t}free(){const e=this.ptr;this.ptr=0,n.__wbg_polygonmap_free(e)}static new(){var e=n.polygonmap_new();return P.__wrap(e)}addPolygon(e,t){var r=a(t,n.__wbindgen_malloc),_=g;n.polygonmap_addPolygon(this.ptr,e,r,_)}rmPolygon(e){n.polygonmap_rmPolygon(this.ptr,e)}polygonContaining(e,t){n.polygonmap_polygonContaining(8,this.ptr,e,t);var r,_,o=b()[2],s=b()[3],u=(r=o,_=s,(null!==p&&p.buffer===n.memory.buffer||(p=new Uint32Array(n.memory.buffer)),p).subarray(r/4,r/4+_)).slice();return n.__wbindgen_free(o,4*s),u}}const T=function(e,t){throw new Error((r=e,_=t,o.decode((null!==s&&s.buffer===n.memory.buffer||(s=new Uint8Array(n.memory.buffer)),s).subarray(r,r+_))));var r,_}},8531:(e,t,r)=>{var n=r.w[e.id];e.exports=n,r(821),n[""]()}}]);
//# sourceMappingURL=995.aladin.js.map