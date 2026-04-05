import { a2 as head } from "../../../../chunks/index.js";
import { D as DiaryNav } from "../../../../chunks/DiaryNav.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const crumbs = [
      { label: "Diary", href: "/diary" },
      { label: "Machine Learning" }
    ];
    head("1fx6yn", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Machine Learning - Contrapunk Diary</title>`);
      });
    });
    DiaryNav($$renderer2, { crumbs });
    $$renderer2.push(`<!----> <div class="chapter svelte-1fx6yn"><header class="chapter-header svelte-1fx6yn"><div class="label svelte-1fx6yn">CHAPTER</div> <h1 class="svelte-1fx6yn">Teaching a Model to Identify Guitar Positions</h1> <p class="svelte-1fx6yn">138 positions on a guitar neck. 1,380 audio samples. Five training rounds.
			Three model architectures. The iterative journey that taught us physics
			works better than pattern matching.</p></header> <section class="approach svelte-1fx6yn"><div class="label svelte-1fx6yn">THE APPROACH</div> <div class="steps svelte-1fx6yn"><div class="step svelte-1fx6yn"><div class="step-num svelte-1fx6yn">1</div><div class="step-text svelte-1fx6yn">Train on raw data</div><div class="step-sub svelte-1fx6yn">Establish baseline</div></div> <div class="step svelte-1fx6yn"><div class="step-num svelte-1fx6yn">2</div><div class="step-text svelte-1fx6yn">Change one thing</div><div class="step-sub svelte-1fx6yn">Measure impact</div></div> <div class="step svelte-1fx6yn"><div class="step-num svelte-1fx6yn">3</div><div class="step-text svelte-1fx6yn">Document result</div><div class="step-sub svelte-1fx6yn">Learn what matters</div></div> <div class="step svelte-1fx6yn"><div class="step-num svelte-1fx6yn">4</div><div class="step-text svelte-1fx6yn">Repeat</div><div class="step-sub svelte-1fx6yn">Until production-ready</div></div></div></section> <section class="rounds svelte-1fx6yn"><div class="label svelte-1fx6yn">TRAINING ROUNDS</div> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<p class="loading-text svelte-1fx6yn">Loading results...</p>`);
    }
    $$renderer2.push(`<!--]--></section> <section class="discovery svelte-1fx6yn"><div class="label svelte-1fx6yn">WHAT WE DISCOVERED</div> <p class="svelte-1fx6yn">After five rounds of iterative training, our best model reached 97.3%.
			Then we discovered that a physics-based approach using the <strong class="svelte-1fx6yn">inharmonicity B coefficient</strong> achieves 98.5% with just one
			calibration sample per string. The full story is in <a href="/diary/machine-learning/the-pivot" class="svelte-1fx6yn">The Pivot</a>.</p></section> <section class="tools svelte-1fx6yn"><a class="tool-card svelte-1fx6yn" href="/diary/machine-learning/explore" style="border-color: rgba(255, 51, 136, 0.3);"><div class="tool-label svelte-1fx6yn" style="color: var(--color-accent-magenta);">EXPLORE DATA</div> <div class="tool-desc svelte-1fx6yn">Browse 138 classes, hear samples</div></a> <a class="tool-card svelte-1fx6yn" href="/diary/machine-learning/playground" style="border-color: rgba(0, 204, 170, 0.3);"><div class="tool-label svelte-1fx6yn" style="color: var(--color-accent-teal);">LIVE PLAYGROUND</div> <div class="tool-desc svelte-1fx6yn">Try the model in your browser</div></a></section></div>`);
  });
}
export {
  _page as default
};
