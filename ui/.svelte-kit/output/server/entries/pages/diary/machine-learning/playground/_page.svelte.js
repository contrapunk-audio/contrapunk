import { a2 as head, e as escape_html } from "../../../../../chunks/index.js";
import { D as DiaryNav } from "../../../../../chunks/DiaryNav.js";
import { C as ConceptInline } from "../../../../../chunks/ConceptInline.js";
import "../../../../../chunks/SpectrogramViewer.svelte_svelte_type_style_lang.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const MODEL_PARAMS = {
      nMels: 64,
      nFft: 1024,
      hopLength: 256
    };
    const crumbs = [
      { label: "Diary", href: "/diary" },
      { label: "Machine Learning", href: "/diary/machine-learning" },
      { label: "Playground" }
    ];
    head("18bu9ct", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Live Playground - Contrapunk Diary</title>`);
      });
    });
    DiaryNav($$renderer2, { crumbs });
    $$renderer2.push(`<!----> <div class="playground svelte-18bu9ct"><header class="playground-header svelte-18bu9ct"><div class="label svelte-18bu9ct">INTERACTIVE</div> <h1 class="svelte-18bu9ct">Try It Yourself</h1> <p class="svelte-18bu9ct">Upload a guitar recording and see how the Pure CNN model classifies which string and
			fret produced the sound. Everything runs in your browser \u2014 no server required.</p></header> <section class="input-section svelte-18bu9ct"><div class="label svelte-18bu9ct">INPUT</div> <div class="input-options svelte-18bu9ct"><button class="input-card svelte-18bu9ct"><div class="input-icon svelte-18bu9ct"><svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg></div> <div class="input-title svelte-18bu9ct">Upload Audio</div> <div class="input-desc svelte-18bu9ct">WAV, MP3, OGG, or FLAC file</div> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></button> <div class="input-card input-card-disabled svelte-18bu9ct"><div class="input-icon svelte-18bu9ct"><svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path><path d="M19 10v2a7 7 0 0 1-14 0v-2"></path><line x1="12" y1="19" x2="12" y2="23"></line><line x1="8" y1="23" x2="16" y2="23"></line></svg></div> <div class="input-title svelte-18bu9ct">Record</div> <div class="input-desc svelte-18bu9ct">Coming soon</div> <div class="input-badge svelte-18bu9ct">FUTURE</div></div></div> <input type="file" accept="audio/*" class="hidden-input svelte-18bu9ct"/></section> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> <section class="explainer-section svelte-18bu9ct"><div class="label svelte-18bu9ct">HOW IT WORKS</div> <div class="explainer-grid svelte-18bu9ct"><div class="explainer-card svelte-18bu9ct"><div class="explainer-num svelte-18bu9ct">1</div> <h3 class="svelte-18bu9ct">Audio Input</h3> <p class="svelte-18bu9ct">Your audio file is decoded using the `);
    ConceptInline($$renderer2, {
      term: "Web Audio API",
      children: ($$renderer3) => {
        $$renderer3.push(`<!---->The Web Audio API is a browser API that provides audio processing capabilities.
						It can decode compressed audio formats (MP3, OGG, etc.) into raw PCM samples
						that we can analyze directly in JavaScript.`);
      }
    });
    $$renderer2.push(`<!----> into raw PCM samples at the original sample rate.</p></div> <div class="explainer-card svelte-18bu9ct"><div class="explainer-num svelte-18bu9ct">2</div> <h3 class="svelte-18bu9ct">Mel Spectrogram</h3> <p class="svelte-18bu9ct">The audio is converted into a `);
    ConceptInline($$renderer2, {
      term: "mel spectrogram",
      children: ($$renderer3) => {
        $$renderer3.push(`<!---->A mel spectrogram is a visual representation of audio that shows how energy
						is distributed across frequency bands over time. The "mel" scale warps frequencies
						to match how humans perceive pitch \u2014 compressing high frequencies and expanding
						low ones. This is the same representation used during training.`);
      }
    });
    $$renderer2.push(`<!----> using an FFT window of ${escape_html(MODEL_PARAMS.nFft)} samples, ${escape_html(MODEL_PARAMS.hopLength)}-sample
					hops, and ${escape_html(MODEL_PARAMS.nMels)} mel frequency bins. The result is a 64x94 image
					that the CNN reads like a picture.</p></div> <div class="explainer-card svelte-18bu9ct"><div class="explainer-num svelte-18bu9ct">3</div> <h3 class="svelte-18bu9ct">CNN Classification</h3> <p class="svelte-18bu9ct">The `);
    ConceptInline($$renderer2, {
      term: "Pure CNN",
      children: ($$renderer3) => {
        $$renderer3.push(`<!---->Our Pure CNN is a 4-layer convolutional neural network with global average pooling.
						It has 424,266 parameters and achieves 97.1% accuracy classifying guitar positions.
						The ONNX export is only ~1.7 MB, making it practical for browser inference.`);
      }
    });
    $$renderer2.push(`<!----> processes the spectrogram through 4 convolutional layers, each extracting
					higher-level features. Global average pooling condenses these into a 256-dimensional
					vector, which a final linear layer maps to 138 class probabilities.</p></div> <div class="explainer-card svelte-18bu9ct"><div class="explainer-num svelte-18bu9ct">4</div> <h3 class="svelte-18bu9ct">Result</h3> <p class="svelte-18bu9ct">The output is a probability distribution over all 138 positions
					(6 strings x 23 frets). The top prediction tells you which string and fret
					most likely produced the sound. The model runs entirely in your browser via `);
    ConceptInline($$renderer2, {
      term: "WebAssembly",
      children: ($$renderer3) => {
        $$renderer3.push(`<!---->WebAssembly (WASM) is a binary format that runs at near-native speed in browsers.
						ONNX Runtime compiles its inference engine to WASM, letting us run the CNN model
						without any server. The ~1.7 MB model + runtime loads once and infers in milliseconds.`);
      }
    });
    $$renderer2.push(`<!----> \u2014 no data leaves your device.</p></div></div></section> <section class="tech-section svelte-18bu9ct"><div class="label svelte-18bu9ct">TECHNICAL DETAILS</div> <div class="tech-grid svelte-18bu9ct"><div class="tech-item svelte-18bu9ct"><div class="tech-key svelte-18bu9ct">Model</div> <div class="tech-val svelte-18bu9ct">Pure CNN (4 conv + GAP + linear)</div></div> <div class="tech-item svelte-18bu9ct"><div class="tech-key svelte-18bu9ct">Parameters</div> <div class="tech-val svelte-18bu9ct">424,266</div></div> <div class="tech-item svelte-18bu9ct"><div class="tech-key svelte-18bu9ct">ONNX Opset</div> <div class="tech-val svelte-18bu9ct">18</div></div> <div class="tech-item svelte-18bu9ct"><div class="tech-key svelte-18bu9ct">Model Size</div> <div class="tech-val svelte-18bu9ct">~1.7 MB (ONNX + data)</div></div> <div class="tech-item svelte-18bu9ct"><div class="tech-key svelte-18bu9ct">Input Shape</div> <div class="tech-val svelte-18bu9ct">(1, 1, 64, 94)</div></div> <div class="tech-item svelte-18bu9ct"><div class="tech-key svelte-18bu9ct">Output</div> <div class="tech-val svelte-18bu9ct">138 class logits</div></div> <div class="tech-item svelte-18bu9ct"><div class="tech-key svelte-18bu9ct">Runtime</div> <div class="tech-val svelte-18bu9ct">ONNX Runtime Web (WASM)</div></div> <div class="tech-item svelte-18bu9ct"><div class="tech-key svelte-18bu9ct">Accuracy</div> <div class="tech-val svelte-18bu9ct">97.1% (Round 1)</div></div></div></section></div>`);
  });
}
export {
  _page as default
};
