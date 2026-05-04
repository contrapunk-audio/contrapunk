import { _ as z } from "../chunks/Ct5FWWRu.js";
import { c as O, a as p, f as w } from "../chunks/DPcBYb6Q.js";
import { a as b, o as D } from "../chunks/B_3aKXjl.js";
import { a4 as M, a7 as u, i as e, ag as x, a9 as P, a5 as L, aa as R, ab as C, a2 as T, a3 as I } from "../chunks/CA4AbA-g.js";
import { s as j } from "../chunks/D6kuWOx4.js";
import { i as q } from "../chunks/CStSVi_H.js";
import { b as F } from "../chunks/DhjuCMT9.js";
import { u as h, c as H, a as k, __tla as __tla_0 } from "../chunks/BOyWGBDd.js";
let it, et;
let __tla = Promise.all([
    (()=>{
        try {
            return __tla_0;
        } catch  {}
    })()
]).then(async ()=>{
    let U, W, B, G;
    U = {};
    W = !1;
    B = !0;
    G = async ()=>{
        {
            const s = U?.PUBLIC_POSTHOG_KEY;
            s && (await z(async ()=>{
                const { default: c } = await import("../chunks/ChjP2LqZ.js");
                return {
                    default: c
                };
            }, [], import.meta.url)).default.init(s, {
                api_host: "https://us.i.posthog.com"
            });
        }
    };
    et = Object.freeze(Object.defineProperty({
        __proto__: null,
        load: G,
        prerender: B,
        ssr: W
    }, Symbol.toStringTag, {
        value: "Module"
    }));
    var K = w('<canvas class="particles-canvas svelte-ljratz" aria-hidden="true"></canvas>');
    function N(s, l) {
        M(l, !0);
        const c = 30, d = [
            "rgba(255, 51, 136, 0.2)",
            "rgba(51, 221, 255, 0.2)",
            "rgba(0, 204, 170, 0.2)",
            "rgba(255, 119, 170, 0.15)",
            "rgba(51, 221, 255, 0.12)"
        ];
        let a = L(void 0), r = null, f = [];
        function _(i, o) {
            f = [];
            for(let n = 0; n < c; n++)f.push({
                x: Math.random() * i,
                y: Math.random() * o,
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
            if (!e(a)) return;
            const i = e(a).getContext("2d");
            if (!i) return;
            const o = e(a).width, n = e(a).height;
            i.clearRect(0, 0, o, n);
            for (const t of f)t.x += t.vx, t.y += t.vy, t.x < -5 && (t.x = o + 5), t.x > o + 5 && (t.x = -5), t.y < -5 && (t.y = n + 5), t.y > n + 5 && (t.y = -5), t.opacity += t.opacitySpeed * t.opacityDir, t.opacity >= .8 && (t.opacityDir = -1), t.opacity <= .15 && (t.opacityDir = 1), i.globalAlpha = t.opacity, i.fillStyle = t.color, i.fillRect(Math.round(t.x), Math.round(t.y), Math.round(t.size), Math.round(t.size));
            i.globalAlpha = 1, r = requestAnimationFrame(v);
        }
        function A() {
            e(a) && (e(a).width = window.innerWidth, e(a).height = window.innerHeight, _(e(a).width, e(a).height), r = requestAnimationFrame(v));
        }
        function m() {
            r !== null && (cancelAnimationFrame(r), r = null);
        }
        function y() {
            !e(a) || !h.animationsEnabled || (e(a).width = window.innerWidth, e(a).height = window.innerHeight, _(e(a).width, e(a).height));
        }
        u(()=>(h.animationsEnabled && e(a) ? (A(), window.addEventListener("resize", y)) : m(), ()=>{
                m(), window.removeEventListener("resize", y);
            })), b(()=>{
            m();
        });
        var g = O(), E = x(g);
        {
            var S = (i)=>{
                var o = K();
                F(o, (n)=>R(a, n), ()=>e(a)), p(i, o);
            };
            q(E, (i)=>{
                h.animationsEnabled && i(S);
            });
        }
        p(s, g), P();
    }
    var V = w('<!> <div class="app-shell svelte-12qhfyh"><!></div>', 1);
    it = function(s, l) {
        M(l, !0), u(()=>{
            h.applyMotionPreference();
        }), u(()=>{
            h.detectSystemMotionPreference();
        }), D(()=>{
            H.refresh();
        }), b(()=>{
            k.destroy?.();
        });
        var c = V(), d = x(c);
        N(d, {});
        var a = C(d, 2), r = T(a);
        j(r, ()=>l.children), I(a), p(s, c), P();
    };
});
export { it as component, et as universal, __tla };
