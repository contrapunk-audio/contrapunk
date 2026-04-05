import { a2 as head } from "../../../../../chunks/index.js";
import { D as DiaryNav } from "../../../../../chunks/DiaryNav.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const crumbs = [
      { label: "Diary", href: "/diary" },
      { label: "Machine Learning", href: "/diary/machine-learning" },
      { label: "Round 4" }
    ];
    head("13v4ubx", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Round 4: Goertzel Harmonics - Contrapunk Diary</title>`);
      });
    });
    DiaryNav($$renderer2, { crumbs });
    $$renderer2.push(`<!----> <div class="round-page svelte-13v4ubx"><header class="hero svelte-13v4ubx"><div class="round-label svelte-13v4ubx">ROUND 04</div> <h1 class="svelte-13v4ubx">Goertzel Harmonics</h1> <p class="hero-sub svelte-13v4ubx">Each guitar string has a unique harmonic fingerprint -- the ratio of overtone
			amplitudes relative to the fundamental. We extracted these using the Goertzel
			algorithm and fused them with the spectrogram in a two-branch neural network.
			The result: harmonic features help weaker models on hard strings, but the
			Pure CNN already captures this information from the spectrogram alone.</p></header> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<p class="loading-text svelte-13v4ubx">Loading results...</p>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
