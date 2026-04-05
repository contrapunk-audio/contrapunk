import { _ as w } from "../chunks/CYEXfWnH.js";
import { c as S, a as p, f as M } from "../chunks/eZFB7GD9.js";
import { J as b, a as u, g as i, Q as P, O as x, K as z, M as L, P as D, F as R, G as T } from "../chunks/nVkCXwlp.js";
import { s as C } from "../chunks/CnfdKV5H.js";
import { a as I } from "../chunks/CIumhm75.js";
import { i as F } from "../chunks/Cswa20vR.js";
import { b as H } from "../chunks/BxTRC_am.js";
import { u as h, __tla as __tla_0 } from "../chunks/Bc0lhIg4.js";
let $, Z;
let __tla = Promise.all([
    (()=>{
        try {
            return __tla_0;
        } catch  {}
    })()
]).then(async ()=>{
    let j, q, G;
    j = !1;
    q = !0;
    G = async ()=>{
        try {
            const { PUBLIC_POSTHOG_KEY: l } = await w(async ()=>{
                const { PUBLIC_POSTHOG_KEY: s } = await import("../chunks/CYEXfWnH.js").then((o)=>o.a);
                return {
                    PUBLIC_POSTHOG_KEY: s
                };
            }, [], import.meta.url);
            l && (await w(async ()=>{
                const { default: o } = await import("../chunks/D7tV_28f.js");
                return {
                    default: o
                };
            }, [], import.meta.url)).default.init(l, {
                api_host: "https://us.i.posthog.com"
            });
        } catch  {}
    };
    Z = Object.freeze(Object.defineProperty({
        __proto__: null,
        load: G,
        prerender: q,
        ssr: j
    }, Symbol.toStringTag, {
        value: "Module"
    }));
    var K = M('<canvas class="particles-canvas svelte-ljratz" aria-hidden="true"></canvas>');
    function U(l, s) {
        b(s, !0);
        const o = 30, d = [
            "rgba(255, 51, 136, 0.2)",
            "rgba(51, 221, 255, 0.2)",
            "rgba(0, 204, 170, 0.2)",
            "rgba(255, 119, 170, 0.15)",
            "rgba(51, 221, 255, 0.12)"
        ];
        let a = z(void 0), c = null, m = [];
        function _(e, n) {
            m = [];
            for(let r = 0; r < o; r++)m.push({
                x: Math.random() * e,
                y: Math.random() * n,
                vx: (Math.random() - .5) * .5,
                vy: (Math.random() - .5) * .4,
                size: 1 + Math.random() * 2,
                color: d[Math.floor(Math.random() * d.length)],
                opacity: .3 + Math.random() * .5,
                opacitySpeed: .003 + Math.random() * .005,
                opacityDir: Math.random() > .5 ? 1 : -1
            });
        }
        function v() {
            if (!i(a)) return;
            const e = i(a).getContext("2d");
            if (!e) return;
            const n = i(a).width, r = i(a).height;
            e.clearRect(0, 0, n, r);
            for (const t of m)t.x += t.vx, t.y += t.vy, t.x < -5 && (t.x = n + 5), t.x > n + 5 && (t.x = -5), t.y < -5 && (t.y = r + 5), t.y > r + 5 && (t.y = -5), t.opacity += t.opacitySpeed * t.opacityDir, t.opacity >= .8 && (t.opacityDir = -1), t.opacity <= .15 && (t.opacityDir = 1), e.globalAlpha = t.opacity, e.fillStyle = t.color, e.fillRect(Math.round(t.x), Math.round(t.y), Math.round(t.size), Math.round(t.size));
            e.globalAlpha = 1, c = requestAnimationFrame(v);
        }
        function E() {
            i(a) && (i(a).width = window.innerWidth, i(a).height = window.innerHeight, _(i(a).width, i(a).height), c = requestAnimationFrame(v));
        }
        function f() {
            c !== null && (cancelAnimationFrame(c), c = null);
        }
        function y() {
            !i(a) || !h.animationsEnabled || (i(a).width = window.innerWidth, i(a).height = window.innerHeight, _(i(a).width, i(a).height));
        }
        u(()=>(h.animationsEnabled && i(a) ? (E(), window.addEventListener("resize", y)) : f(), ()=>{
                f(), window.removeEventListener("resize", y);
            })), I(()=>{
            f();
        });
        var g = S(), O = P(g);
        {
            var A = (e)=>{
                var n = K();
                H(n, (r)=>L(a, r), ()=>i(a)), p(e, n);
            };
            F(O, (e)=>{
                h.animationsEnabled && e(A);
            });
        }
        p(l, g), x();
    }
    var B = M('<!> <div class="app-shell svelte-12qhfyh"><!></div>', 1);
    $ = function(l, s) {
        b(s, !0), u(()=>{
            h.applyMotionPreference();
        }), u(()=>{
            h.detectSystemMotionPreference();
        });
        var o = B(), d = P(o);
        U(d, {});
        var a = D(d, 2), c = R(a);
        C(c, ()=>s.children), T(a), p(l, o), x();
    };
});
export { $ as component, Z as universal, __tla };
