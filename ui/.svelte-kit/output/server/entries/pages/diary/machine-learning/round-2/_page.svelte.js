import { a2 as head } from "../../../../../chunks/index.js";
import { D as DiaryNav } from "../../../../../chunks/DiaryNav.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const crumbs = [
      { label: "Diary", href: "/diary" },
      { label: "Machine Learning", href: "/diary/machine-learning" },
      { label: "Round 2" }
    ];
    head("bso89z", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Round 2: Onset Alignment - Contrapunk Diary</title>`);
      });
    });
    DiaryNav($$renderer2, { crumbs });
    $$renderer2.push(`<!----> <div class="round-page svelte-bso89z"><header class="hero svelte-bso89z"><div class="round-label svelte-bso89z">ROUND 2</div> <h1 class="svelte-bso89z">Onset Alignment</h1> <p class="hero-sub svelte-bso89z">Hypothesis: every audio sample has some silence before the pluck. If we detect
			the exact moment the string starts vibrating and trim everything before it,
			the spectrogram should be cleaner and the classifier should perform better.
			Result: zero measurable change. Here is why that is actually informative.</p></header> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<p class="loading-text svelte-bso89z">Loading results...</p>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
