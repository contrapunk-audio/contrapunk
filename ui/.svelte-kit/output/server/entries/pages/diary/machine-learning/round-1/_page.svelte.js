import { a2 as head } from "../../../../../chunks/index.js";
import { D as DiaryNav } from "../../../../../chunks/DiaryNav.js";
import "../../../../../chunks/SpectrogramViewer.svelte_svelte_type_style_lang.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const crumbs = [
      { label: "Diary", href: "/diary" },
      { label: "Machine Learning", href: "/diary/machine-learning" },
      { label: "Round 1" }
    ];
    head("jkbg0s", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Round 1: Raw Baseline - Contrapunk Diary</title>`);
      });
    });
    DiaryNav($$renderer2, { crumbs });
    $$renderer2.push(`<!----> <div class="round-page svelte-jkbg0s"><header class="hero svelte-jkbg0s"><div class="round-label svelte-jkbg0s">ROUND 1</div> <h1 class="svelte-jkbg0s">Raw Baseline</h1> <p class="hero-sub svelte-jkbg0s">No preprocessing. No augmentation. Raw audio turned into mel-spectrograms
			and fed into three different classifiers. The goal: establish what accuracy
			we get with zero effort on the data pipeline.</p></header> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<p class="loading-text svelte-jkbg0s">Loading results...</p>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
