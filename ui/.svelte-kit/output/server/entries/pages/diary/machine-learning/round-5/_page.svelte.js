import { a2 as head } from "../../../../../chunks/index.js";
import { D as DiaryNav } from "../../../../../chunks/DiaryNav.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const crumbs = [
      { label: "Diary", href: "/diary" },
      { label: "Machine Learning", href: "/diary/machine-learning" },
      { label: "Round 5" }
    ];
    head("1d9wa8o", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Round 5: Data Augmentation - Contrapunk Diary</title>`);
      });
    });
    DiaryNav($$renderer2, { crumbs });
    $$renderer2.push(`<!----> <div class="round-page svelte-1d9wa8o"><header class="hero svelte-1d9wa8o"><div class="round-label svelte-1d9wa8o">ROUND 05</div> <h1 class="svelte-1d9wa8o">Data Augmentation</h1> <p class="hero-sub svelte-1d9wa8o">With only 10 samples per class, the models are starved for data. We generated
			3 augmented copies per sample -- gain variation, noise injection, time shift,
			time stretch -- growing the training set from 1,380 to 5,520 samples. The Hybrid
			CNN hit a new personal best (95.1%), but the Pure CNN stayed flat. Round 3's
			97.3% remains the overall best.</p></header> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<p class="loading-text svelte-1d9wa8o">Loading results...</p>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
