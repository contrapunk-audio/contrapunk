import { a2 as head } from "../../../../../chunks/index.js";
import { D as DiaryNav } from "../../../../../chunks/DiaryNav.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const crumbs = [
      { label: "Diary", href: "/diary" },
      { label: "Machine Learning", href: "/diary/machine-learning" },
      { label: "Round 3" }
    ];
    head("iy1wju", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Round 3: Quality Cleanup - Contrapunk Diary</title>`);
      });
    });
    DiaryNav($$renderer2, { crumbs });
    $$renderer2.push(`<!----> <div class="round-page svelte-iy1wju"><header class="hero svelte-iy1wju"><div class="round-label svelte-iy1wju">ROUND 03</div> <h1 class="svelte-iy1wju">Quality Cleanup</h1> <p class="hero-sub svelte-iy1wju">The first round with measurable improvement. Removed 32 clipped samples and
			7 near-silent recordings from 1,380 total, leaving 1,341 clean samples.
			Data quality beats data quantity -- removing 39 bad samples helped more than
			aligning 1,380.</p></header> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<p class="loading-text svelte-iy1wju">Loading results...</p>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
