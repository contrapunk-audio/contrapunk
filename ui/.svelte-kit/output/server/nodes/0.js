

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": true,
  "load": null
};
export const universal_id = "src/routes/+layout.ts";
export const imports = ["_app/immutable/nodes/0.CnP1nWho.js","_app/immutable/chunks/CYEXfWnH.js","_app/immutable/chunks/eZFB7GD9.js","_app/immutable/chunks/nVkCXwlp.js","_app/immutable/chunks/CnfdKV5H.js","_app/immutable/chunks/C_o2MEX2.js","_app/immutable/chunks/CIumhm75.js","_app/immutable/chunks/Cswa20vR.js","_app/immutable/chunks/BxTRC_am.js","_app/immutable/chunks/Bc0lhIg4.js"];
export const stylesheets = ["_app/immutable/assets/0.DGR_DYs0.css"];
export const fonts = [];
