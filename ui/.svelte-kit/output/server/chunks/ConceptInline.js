import { e as escape_html } from "./index.js";
import "clsx";
function ConceptInline($$renderer, $$props) {
  let { term, children } = $$props;
  $$renderer.push(`<span class="concept-trigger svelte-vur0pt">${escape_html(term)}</span> `);
  {
    $$renderer.push("<!--[!-->");
  }
  $$renderer.push(`<!--]-->`);
}
export {
  ConceptInline as C
};
