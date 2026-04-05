import { Z as ensure_array_like, $ as attr_style, e as escape_html, a0 as stringify, _ as attr_class, Y as derived, a2 as head, a1 as attr } from "../../../chunks/index.js";
import { D as DiaryNav } from "../../../chunks/DiaryNav.js";
import "../../../chunks/SpectrogramViewer.svelte_svelte_type_style_lang.js";
function StatBar($$renderer, $$props) {
  let { stats = [] } = $$props;
  $$renderer.push(`<div class="stat-bar svelte-1vx9ofr"><!--[-->`);
  const each_array = ensure_array_like(stats);
  for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
    let stat = each_array[$$index];
    $$renderer.push(`<div class="stat svelte-1vx9ofr"><div class="stat-value svelte-1vx9ofr"${attr_style("", { color: stat.color || "var(--color-accent-cyan)" })}>${escape_html(stat.value)}</div> <div class="stat-label svelte-1vx9ofr">${escape_html(stat.label)}</div></div>`);
  }
  $$renderer.push(`<!--]--></div>`);
}
function WaveformDisplay($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { height = 64 } = $$props;
    $$renderer2.push(`<div class="waveform-container svelte-1l4jjmr"${attr_style(`height: ${stringify(
      // Decode audio and downsample to peaks
      // Draw waveform and playhead
      // Background
      // Waveform bars
      // Played portion — brighter cyan
      // Unplayed portion — dim cyan
      // Playhead line
      height
    )}px`)}>`);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="waveform-loading svelte-1l4jjmr">Loading waveform...</div>`);
    }
    $$renderer2.push(`<!--]--> <canvas class="waveform-canvas svelte-1l4jjmr"${attr_style(`height: ${stringify(height)}px`)}></canvas></div>`);
  });
}
function DemoAnimation($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const strings = [
      { key: "E2", label: "E2 open", confidence: "97.2%" },
      { key: "A2", label: "A2 open", confidence: "96.8%" },
      { key: "D3", label: "D3 open", confidence: "98.1%" },
      { key: "G3", label: "G3 open", confidence: "95.4%" },
      { key: "B3", label: "B3 open", confidence: "97.6%" },
      { key: "E4", label: "E4 open", confidence: "98.5%" }
    ];
    let selected = "E2";
    let stage = 0;
    let audioUrl = derived(() => `/samples/showcase/${selected}_open.wav`);
    let info = derived(() => strings.find((s) => s.key === selected));
    $$renderer2.push(`<div class="demo-wrap svelte-8z7z47"><div${attr_class("demo-pipeline svelte-8z7z47", void 0, {
      "active": (
        // Cleanup on destroy
        stage >= 1
      )
    })}><div${attr_class("demo-stage waveform-stage svelte-8z7z47", void 0, { "visible": stage >= 1 })}><div class="stage-label svelte-8z7z47">WAVEFORM</div> `);
    WaveformDisplay($$renderer2, { url: audioUrl(), height: 56 });
    $$renderer2.push(`<!----></div> <div${attr_class("demo-arrow svelte-8z7z47", void 0, { "visible": stage >= 2 })}><svg width="32" height="20" viewBox="0 0 32 20" aria-hidden="true"><line x1="0" y1="10" x2="24" y2="10" stroke="var(--color-accent-cyan)" stroke-width="2"></line><polygon points="22,5 32,10 22,15" fill="var(--color-accent-cyan)"></polygon></svg></div> <div${attr_class("demo-stage spectrogram-stage svelte-8z7z47", void 0, { "visible": stage >= 2 })}><div class="stage-label svelte-8z7z47">SPECTROGRAM</div> `);
    {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<div class="stage-placeholder svelte-8z7z47" style="height: 56px"></div>`);
    }
    $$renderer2.push(`<!--]--></div> <div${attr_class("demo-arrow svelte-8z7z47", void 0, { "visible": stage >= 3 })}><svg width="32" height="20" viewBox="0 0 32 20" aria-hidden="true"><line x1="0" y1="10" x2="24" y2="10" stroke="var(--color-accent-cyan)" stroke-width="2"></line><polygon points="22,5 32,10 22,15" fill="var(--color-accent-cyan)"></polygon></svg></div> <div${attr_class("demo-stage result-stage svelte-8z7z47", void 0, { "visible": stage >= 4 })}><div class="stage-label svelte-8z7z47">RESULT</div> <div class="result-card svelte-8z7z47"><div class="result-note svelte-8z7z47">${escape_html(info().label)}</div> <div class="result-confidence svelte-8z7z47">${escape_html(info().confidence)}</div> <div class="confidence-bar svelte-8z7z47"><div class="confidence-fill svelte-8z7z47"${attr_style(`width: ${stringify("0%")}`)}></div></div></div></div></div> <div class="demo-controls svelte-8z7z47">`);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<button class="pixel-btn play-demo-btn svelte-8z7z47">PLAY DEMO</button>`);
    }
    $$renderer2.push(`<!--]--></div> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
function _page($$renderer) {
  const chapters = [
    {
      phase: "01-03",
      title: "MIDI Foundation",
      desc: "Input, harmony engine, multi-port output",
      complete: true
    },
    {
      phase: "04-05",
      title: "Network and WASM",
      desc: "Server mode and browser deployment",
      complete: true
    },
    {
      phase: "06",
      title: "Harmony and UI",
      desc: "Modal harmony, chord detection, HLD pixel art UI",
      complete: true
    },
    {
      phase: "08",
      title: "Machine Learning",
      desc: "Guitar string+fret classifier",
      complete: false,
      active: true,
      href: "/diary/machine-learning"
    },
    {
      phase: "09",
      title: "Vocoder",
      desc: "Real-time vocal harmonization",
      complete: false
    },
    {
      phase: "10",
      title: "Guitar Input",
      desc: "Live audio to MIDI via ML classifier",
      complete: false
    }
  ];
  const stats = [
    {
      value: "96.2%",
      label: "Accuracy",
      color: "var(--color-accent-cyan)"
    },
    {
      value: "2.1ms",
      label: "Inference",
      color: "var(--color-accent-teal)"
    },
    {
      value: "138",
      label: "Classes",
      color: "var(--color-accent-magenta)"
    },
    {
      value: "0",
      label: "Dependencies",
      color: "var(--color-text-primary)"
    }
  ];
  head("1uyldbo", $$renderer, ($$renderer2) => {
    $$renderer2.title(($$renderer3) => {
      $$renderer3.push(`<title>Diary - Contrapunk</title>`);
    });
  });
  DiaryNav($$renderer, { crumbs: [{ label: "Diary" }] });
  $$renderer.push(`<!----> <div class="landing svelte-1uyldbo"><header class="hero svelte-1uyldbo"><div class="tagline-label svelte-1uyldbo">CONTRAPUNK</div> <h1 class="svelte-1uyldbo">An Improvisation Companion</h1> <p class="subtitle svelte-1uyldbo">From raw audio to real-time MIDI. The open-source journey of building
			a tool that understands your instrument and harmonizes with you live.</p></header> <div class="demo-section svelte-1uyldbo">`);
  DemoAnimation($$renderer);
  $$renderer.push(`<!----></div> `);
  StatBar($$renderer, { stats });
  $$renderer.push(`<!----> <section class="chapters svelte-1uyldbo"><div class="section-label svelte-1uyldbo">THE JOURNEY</div> <div class="chapter-grid svelte-1uyldbo"><!--[-->`);
  const each_array = ensure_array_like(chapters);
  for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
    let ch = each_array[$$index];
    if (ch.href) {
      $$renderer.push("<!--[-->");
      $$renderer.push(`<a${attr_class("chapter-card svelte-1uyldbo", void 0, { "active": ch.active })}${attr("href", ch.href)}><div class="chapter-phase svelte-1uyldbo">${escape_html(ch.phase)}</div> <div class="chapter-title svelte-1uyldbo">${escape_html(ch.title)}</div> <div class="chapter-desc svelte-1uyldbo">${escape_html(ch.desc)}</div> `);
      if (ch.active) {
        $$renderer.push("<!--[-->");
        $$renderer.push(`<div class="chapter-status active svelte-1uyldbo">Active</div>`);
      } else {
        $$renderer.push("<!--[!-->");
      }
      $$renderer.push(`<!--]--></a>`);
    } else {
      $$renderer.push("<!--[!-->");
      $$renderer.push(`<div${attr_class("chapter-card svelte-1uyldbo", void 0, { "complete": ch.complete, "future": !ch.complete })}><div class="chapter-phase svelte-1uyldbo">${escape_html(ch.phase)}</div> <div class="chapter-title svelte-1uyldbo">${escape_html(ch.title)}</div> <div class="chapter-desc svelte-1uyldbo">${escape_html(ch.desc)}</div> `);
      if (ch.complete) {
        $$renderer.push("<!--[-->");
        $$renderer.push(`<div class="chapter-status complete svelte-1uyldbo">Complete</div>`);
      } else {
        $$renderer.push("<!--[!-->");
      }
      $$renderer.push(`<!--]--></div>`);
    }
    $$renderer.push(`<!--]-->`);
  }
  $$renderer.push(`<!--]--></div></section></div>`);
}
export {
  _page as default
};
