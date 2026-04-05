import "clsx";
import { u as ui } from "../../chunks/ui.svelte.js";
import { s as ssr_context } from "../../chunks/context.js";
import "@tauri-apps/api/core";
import "@tauri-apps/api/event";
function onDestroy(fn) {
  /** @type {SSRContext} */
  ssr_context.r.on_destroy(fn);
}
function Particles($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    onDestroy(() => {
    });
    if (ui.animationsEnabled) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<canvas class="particles-canvas svelte-ljratz" aria-hidden="true"></canvas>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]-->`);
  });
}
function _layout($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { children } = $$props;
    Particles($$renderer2);
    $$renderer2.push(`<!----> <div class="app-shell svelte-12qhfyh">`);
    children($$renderer2);
    $$renderer2.push(`<!----></div>`);
  });
}
export {
  _layout as default
};
