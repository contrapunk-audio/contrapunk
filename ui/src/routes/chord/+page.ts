// Chord mini app — standalone chord-detection page (#12).
//
// SSR disabled because the page depends on the WASM engine which only
// boots client-side. Prerender on so the page is statically built into
// the deploy artifact and loads instantly from Cloudflare Pages.

export const ssr = false;
export const prerender = true;
