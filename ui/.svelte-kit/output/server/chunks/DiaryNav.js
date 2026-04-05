import { Z as ensure_array_like, a1 as attr, e as escape_html } from "./index.js";
function DiaryNav($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { crumbs = [] } = $$props;
    $$renderer2.push(`<nav class="diary-nav svelte-1elfjn0"><a href="/" class="brand svelte-1elfjn0">CONTRAPUNK</a> <!--[-->`);
    const each_array = ensure_array_like(crumbs);
    for (let i = 0, $$length = each_array.length; i < $$length; i++) {
      let crumb = each_array[i];
      $$renderer2.push(`<span class="sep svelte-1elfjn0">></span> `);
      if (crumb.href && i < crumbs.length - 1) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<a${attr("href", crumb.href)} class="crumb svelte-1elfjn0">${escape_html(crumb.label)}</a>`);
      } else {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`<span class="crumb active svelte-1elfjn0">${escape_html(crumb.label)}</span>`);
      }
      $$renderer2.push(`<!--]-->`);
    }
    $$renderer2.push(`<!--]--></nav>`);
  });
}
export {
  DiaryNav as D
};
