import { a2 as head } from "../../../../../chunks/index.js";
import { D as DiaryNav } from "../../../../../chunks/DiaryNav.js";
import { C as ConceptInline } from "../../../../../chunks/ConceptInline.js";
function _page($$renderer) {
  const crumbs = [
    { label: "Diary", href: "/diary" },
    { label: "Machine Learning", href: "/diary/machine-learning" },
    { label: "The Pivot" }
  ];
  head("16x2xvs", $$renderer, ($$renderer2) => {
    $$renderer2.title(($$renderer3) => {
      $$renderer3.push(`<title>The Pivot - Contrapunk Diary</title>`);
    });
  });
  DiaryNav($$renderer, { crumbs });
  $$renderer.push(`<!----> <div class="pivot-page svelte-16x2xvs"><header class="hero svelte-16x2xvs"><div class="section-label svelte-16x2xvs">CONCLUSION</div> <h1 class="svelte-16x2xvs">The Pivot</h1> <p class="hero-sub svelte-16x2xvs">What five rounds of training taught us \u2014 and what a research paper changed.</p></header> <section class="station svelte-16x2xvs"><div class="station-label svelte-16x2xvs">THE JOURNEY SO FAR</div> <h2 class="svelte-16x2xvs">Five Rounds, One Ceiling</h2> <p class="svelte-16x2xvs">Each training round changed one variable and measured the impact. The results
			told a clear story.</p> <div class="journey-table-wrap svelte-16x2xvs"><table class="journey-table svelte-16x2xvs"><thead><tr><th class="svelte-16x2xvs">Round</th><th class="svelte-16x2xvs">Change</th><th class="svelte-16x2xvs">Pure CNN</th><th class="svelte-16x2xvs">Lesson</th></tr></thead><tbody><tr><td class="round-num svelte-16x2xvs">1</td><td class="svelte-16x2xvs">Raw baseline</td><td class="acc svelte-16x2xvs">96.2%</td><td class="svelte-16x2xvs">Mel-spectrograms work</td></tr><tr><td class="round-num svelte-16x2xvs">2</td><td class="svelte-16x2xvs">Onset alignment</td><td class="acc svelte-16x2xvs">96.2%</td><td class="svelte-16x2xvs">Alignment doesn't matter</td></tr><tr class="highlight-row svelte-16x2xvs"><td class="round-num svelte-16x2xvs">3</td><td class="svelte-16x2xvs">Quality cleanup</td><td class="acc best svelte-16x2xvs">97.3%</td><td class="svelte-16x2xvs">Clean data > more data</td></tr><tr><td class="round-num svelte-16x2xvs">4</td><td class="svelte-16x2xvs">Goertzel harmonics</td><td class="acc svelte-16x2xvs">96.4%</td><td class="svelte-16x2xvs">Extra features hurt</td></tr><tr><td class="round-num svelte-16x2xvs">5</td><td class="svelte-16x2xvs">3x augmentation</td><td class="acc svelte-16x2xvs">96.4%</td><td class="svelte-16x2xvs">More data doesn't help</td></tr></tbody></table></div> <p class="callout svelte-16x2xvs">The pattern was clear: nothing we tried could break through 97%.</p></section> <section class="station svelte-16x2xvs"><div class="station-label svelte-16x2xvs">THE CEILING</div> <h2 class="svelte-16x2xvs">Why 97% Is a Wall</h2> <p class="svelte-16x2xvs">The accuracy plateau is not a training problem. It is a physics problem.
			Three factors create a hard ceiling for any spectrogram-based classifier
			working with this dataset:</p> <ul class="findings svelte-16x2xvs"><li class="svelte-16x2xvs"><span class="finding-icon num svelte-16x2xvs">1</span> <div><strong class="svelte-16x2xvs">10 samples per class is fundamentally limiting for a 138-class problem.</strong> With 138 string+fret positions and only 10 examples of each, the model sees
					minimal variation per class. It cannot learn the full range of how a given
					position sounds across different playing dynamics, pick positions, and
					string ages. Any statistical classifier needs more examples to reliably
					distinguish classes that are acoustically similar.</div></li> <li class="svelte-16x2xvs"><span class="finding-icon num svelte-16x2xvs">2</span> <div><strong class="svelte-16x2xvs">Mel-spectrograms capture frequency content but lose phase information.</strong> The `);
  {
    let children = function($$renderer2) {
      $$renderer2.push(`<!---->A mel-spectrogram is a visual representation of audio that shows frequency
							content over time, using the mel scale (a perceptual scale that spaces
							frequencies the way humans hear them). It captures what frequencies are
							present and how loud they are, but discards phase information \u2014 the
							timing relationships between frequency components. Phase carries subtle
							cues about the physical source of a sound.`);
    };
    ConceptInline($$renderer, {
      term: "mel-spectrogram",
      children
    });
  }
  $$renderer.push(`<!----> tells us what frequencies are present and how loud they are, but not the
					precise timing relationships between harmonics. Those timing relationships
					\u2014 the phase structure \u2014 carry information about the physical source of the
					sound that could help distinguish strings.</div></li> <li class="svelte-16x2xvs"><span class="finding-icon num svelte-16x2xvs">3</span> <div><strong class="svelte-16x2xvs">37 same-note-different-string pairs share the same fundamental frequency.</strong> A guitar can produce the same pitch on multiple strings at different frets.
					For example, the note A3 (220 Hz) can be played on the 5th fret of the low E
					string, the open A string, or the 7th fret of the D string. The fundamental
					frequency is identical \u2014 the model must rely on subtle harmonic differences
					between strings, and 10 samples per class is not enough to learn those
					differences reliably.</div></li></ul></section> <section class="station svelte-16x2xvs"><div class="station-label svelte-16x2xvs">THE RESEARCH</div> <h2 class="svelte-16x2xvs">A 2019 Paper Changed Everything</h2> <p class="svelte-16x2xvs">While investigating why our CNN plateaued on same-note pairs, we read the
			literature on guitar string identification. A paper by Hjerrild and Christensen
			(2019) described an approach based on the physics of vibrating strings \u2014 the `);
  {
    let children = function($$renderer2) {
      $$renderer2.push(`<!---->The inharmonicity coefficient B is a dimensionless number that describes
					how much the harmonics of a vibrating string deviate from perfect integer
					multiples of the fundamental. An ideal string would have harmonics at
					exactly 2x, 3x, 4x the fundamental. Real strings, because they have
					stiffness, produce harmonics that are slightly sharp \u2014 and the deviation
					grows with harmonic number. B depends on the string's physical properties:
					diameter, tension, density, and vibrating length. Each string on a guitar
					has a unique B value.`);
    };
    ConceptInline($$renderer, {
      term: "inharmonicity B coefficient",
      children
    });
  }
  $$renderer.push(`<!---->.</p> <div class="equation-box svelte-16x2xvs"><div class="equation-label svelte-16x2xvs">THE INHARMONICITY MODEL</div> <div class="equation svelte-16x2xvs"><span class="eq-part svelte-16x2xvs">f<sub>n</sub></span> <span class="eq-op svelte-16x2xvs">=</span> <span class="eq-part svelte-16x2xvs">n</span> <span class="eq-op svelte-16x2xvs">\xB7</span> <span class="eq-part svelte-16x2xvs">f<sub>1</sub></span> <span class="eq-op svelte-16x2xvs">\xB7</span> <span class="eq-part svelte-16x2xvs">\u221A(1 + B \xB7 n<sup>2</sup>)</span></div> <div class="equation-legend svelte-16x2xvs"><span><em class="svelte-16x2xvs">f<sub>n</sub></em> = frequency of the nth harmonic</span> <span><em class="svelte-16x2xvs">n</em> = harmonic number (1, 2, 3...)</span> <span><em class="svelte-16x2xvs">f<sub>1</sub></em> = fundamental frequency</span> <span><em class="svelte-16x2xvs">B</em> = inharmonicity coefficient (unique per string)</span></div></div> <p class="svelte-16x2xvs">The key insight: `);
  {
    let children = function($$renderer2) {
      $$renderer2.push(`<!---->The B coefficient is determined by the string's diameter, tension, density,
					and vibrating length. When you press a different fret, you change the
					vibrating length \u2014 but B stays roughly constant because the ratio of
					stiffness to tension doesn't change significantly. This means B identifies
					the string regardless of which fret is pressed. Once you know the pitch
					(from any standard pitch detector) and the string (from B), the fret is
					trivially computed.`);
    };
    ConceptInline($$renderer, {
      term: "B depends on string properties, not fret position",
      children
    });
  }
  $$renderer.push(`<!---->.
			Each string has a unique B regardless of fret. By measuring the deviation of
			the first 6 harmonics from perfect integer multiples of the fundamental,
			you can compute B and identify which string is vibrating \u2014 with 98.5% accuracy
			using just 1 calibration sample per string.</p> <p class="callout svelte-16x2xvs">The physics already solved this problem. We were trying to learn with neural
			networks what string theory already describes with an equation.</p></section> <section class="station svelte-16x2xvs"><div class="station-label svelte-16x2xvs">THE COMPARISON</div> <h2 class="svelte-16x2xvs">DSP vs ML \u2014 Side by Side</h2> <p class="svelte-16x2xvs">The numbers make the case. A physics-based approach outperforms our best
			trained model on every metric that matters for a real-time instrument.</p> <div class="compare-table-wrap svelte-16x2xvs"><table class="compare-table svelte-16x2xvs"><thead><tr><th class="svelte-16x2xvs">Aspect</th><th class="svelte-16x2xvs">ML (our CNN)</th><th class="dsp-col svelte-16x2xvs">DSP (inharmonicity)</th></tr></thead><tbody><tr><td class="aspect-label svelte-16x2xvs">String+fret accuracy</td><td class="svelte-16x2xvs">97.3%</td><td class="dsp-col winner svelte-16x2xvs">98.5%</td></tr><tr><td class="aspect-label svelte-16x2xvs">Training data needed</td><td class="svelte-16x2xvs">1,380 samples</td><td class="dsp-col winner svelte-16x2xvs">6 samples (1/string)</td></tr><tr><td class="aspect-label svelte-16x2xvs">Latency</td><td class="svelte-16x2xvs">~50ms</td><td class="dsp-col winner svelte-16x2xvs">~40ms</td></tr><tr><td class="aspect-label svelte-16x2xvs">Pitch bends</td><td class="limitation svelte-16x2xvs">Cannot track</td><td class="dsp-col winner svelte-16x2xvs">Frame-by-frame</td></tr><tr><td class="aspect-label svelte-16x2xvs">Legato</td><td class="limitation svelte-16x2xvs">Cannot detect</td><td class="dsp-col winner svelte-16x2xvs">Detects pitch without onset</td></tr></tbody></table></div></section> <section class="station svelte-16x2xvs"><div class="station-label svelte-16x2xvs">THE PLAN</div> <h2 class="svelte-16x2xvs">The Hybrid Architecture</h2> <p class="svelte-16x2xvs">We are not throwing away the ML work. The plan is to use both approaches
			where each excels \u2014 DSP for speed and physics-grounded accuracy, ML for
			edge-case refinement.</p> <div class="architecture-box svelte-16x2xvs"><div class="arch-label svelte-16x2xvs">HYBRID PIPELINE</div> <div class="arch-flow svelte-16x2xvs"><div class="arch-node input svelte-16x2xvs">Audio In</div> <div class="arch-split svelte-16x2xvs"><div class="arch-branch fast svelte-16x2xvs"><div class="arch-branch-label svelte-16x2xvs">FAST PATH (~8ms)</div> <div class="arch-node svelte-16x2xvs">Onset + Pitch (DSP)</div> <div class="arch-result svelte-16x2xvs">Instant MIDI note</div></div> <div class="arch-branch slow svelte-16x2xvs"><div class="arch-branch-label svelte-16x2xvs">SLOW PATH (~50ms)</div> <div class="arch-node svelte-16x2xvs">CNN Classifier (ML)</div> <div class="arch-result svelte-16x2xvs">String confirmation</div></div></div> <div class="arch-node output svelte-16x2xvs">Tablature-accurate MIDI with string routing</div></div></div> <p class="svelte-16x2xvs">The fast DSP path detects the onset, estimates the pitch, and sends a MIDI
			note within 8ms \u2014 fast enough for a player to feel no latency. The slow ML
			path runs the CNN classifier in parallel and, 50ms later, either confirms
			the string assignment or corrects it. The result is both fast and accurate.</p></section> <section class="station svelte-16x2xvs"><div class="station-label svelte-16x2xvs">LESSONS</div> <h2 class="svelte-16x2xvs">What the ML Journey Taught Us</h2> <ul class="learnings svelte-16x2xvs"><li class="svelte-16x2xvs"><strong class="svelte-16x2xvs">Data quality matters more than quantity.</strong> Round 3 (removing 39 bad samples) outperformed Round 5 (tripling the
				dataset through augmentation). Cleaning the data gave us +0.9%.
				Augmenting it gave us +0.0%.</li> <li class="svelte-16x2xvs"><strong class="svelte-16x2xvs">Simple architectures beat complex ones on small data.</strong> The Pure CNN consistently outperformed the Hybrid fusion model. With
				only 10 samples per class, a simpler model generalizes better \u2014 it has
				fewer parameters to overfit.</li> <li class="svelte-16x2xvs"><strong class="svelte-16x2xvs">The iterative approach was valuable.</strong> Each round revealed something specific. Round 2 told us the capture tool
				was already good. Round 3 told us data quality matters. Round 4 told us
				the CNN already learns harmonics implicitly. Round 5 confirmed the
				ceiling is physics-bounded, not data-bounded.</li> <li class="svelte-16x2xvs"><strong class="svelte-16x2xvs">The journey to ML led us to understand the physics.</strong> Investigating why Goertzel harmonics did not help the CNN led us to the
				inharmonicity literature. The failure of Round 4 pointed directly to
				the solution.</li> <li class="svelte-16x2xvs"><strong class="svelte-16x2xvs">Sometimes the best outcome of a machine learning project is discovering you don't need machine learning.</strong> Five rounds of training, 1,380 carefully captured samples, three model
				architectures \u2014 and the most important result was understanding the
				problem well enough to find a better tool. The ML journey was not wasted.
				It was the path to the answer.</li></ul></section> <section class="station last svelte-16x2xvs"><div class="station-label svelte-16x2xvs">WHAT'S NEXT</div> <h2 class="svelte-16x2xvs">The DSP Pipeline</h2> <p class="svelte-16x2xvs">The next chapter implements the physics-based approach: pitch detection with
			YIN autocorrelation, inharmonicity measurement from the first 6 harmonics,
			string identification via B coefficient lookup, and continuous pitch tracking
			for bends and vibrato.</p> <div class="nav-links svelte-16x2xvs"><a href="/diary/machine-learning/round-3" class="nav-link prev svelte-16x2xvs">&lt;- Round 3: Quality Cleanup</a> <a href="/diary/machine-learning" class="nav-link next svelte-16x2xvs">Back to Overview -></a></div></section></div>`);
}
export {
  _page as default
};
