import { clsx as clsx$1 } from "clsx";
import { b as set_ssr_context, s as ssr_context, p as push, c as pop } from "./context.js";
import * as devalue from "devalue";
let attr_style, ASYNC, BOUNDARY_EFFECT, COMMENT_NODE, DIRTY, ERROR_VALUE, get_descriptor, get_prototype_of, HYDRATION_ERROR, INERT, is_array, is_extensible, USER_EFFECT, MAYBE_DIRTY, REACTION_IS_UPDATING, index_of, define_property, array_from, REACTION_RAN, STALE_REACTION, is_passive_event, UNINITIALIZED, LEGACY_PROPS, WAS_MARKED, render, derived, ensure_array_like, attr_class, HYDRATION_END, stringify, attr, head, HYDRATION_START, HYDRATION_START_ELSE, EFFECT, escape_html, CONNECTED, CLEAN, DERIVED, BLOCK_EFFECT, deferred, BRANCH_EFFECT, ROOT_EFFECT, RENDER_EFFECT, noop, MANAGED_EFFECT, HEAD_EFFECT, DESTROYED, run_all, includes, HYDRATION_START_FAILED, EFFECT_TRANSPARENT, EFFECT_PRESERVED, EAGER_EFFECT, STATE_SYMBOL, object_prototype, array_prototype;
let __tla = (async () => {
  is_array = Array.isArray;
  index_of = Array.prototype.indexOf;
  includes = Array.prototype.includes;
  array_from = Array.from;
  define_property = Object.defineProperty;
  get_descriptor = Object.getOwnPropertyDescriptor;
  object_prototype = Object.prototype;
  array_prototype = Array.prototype;
  get_prototype_of = Object.getPrototypeOf;
  is_extensible = Object.isExtensible;
  var has_own_property = Object.prototype.hasOwnProperty;
  noop = () => {
  };
  run_all = function(arr) {
    for (var i = 0; i < arr.length; i++) {
      arr[i]();
    }
  };
  deferred = function() {
    var resolve;
    var reject;
    var promise = new Promise((res, rej) => {
      resolve = res;
      reject = rej;
    });
    return {
      promise,
      resolve,
      reject
    };
  };
  DERIVED = 1 << 1;
  EFFECT = 1 << 2;
  RENDER_EFFECT = 1 << 3;
  MANAGED_EFFECT = 1 << 24;
  BLOCK_EFFECT = 1 << 4;
  BRANCH_EFFECT = 1 << 5;
  ROOT_EFFECT = 1 << 6;
  BOUNDARY_EFFECT = 1 << 7;
  CONNECTED = 1 << 9;
  CLEAN = 1 << 10;
  DIRTY = 1 << 11;
  MAYBE_DIRTY = 1 << 12;
  INERT = 1 << 13;
  DESTROYED = 1 << 14;
  REACTION_RAN = 1 << 15;
  EFFECT_TRANSPARENT = 1 << 16;
  EAGER_EFFECT = 1 << 17;
  HEAD_EFFECT = 1 << 18;
  EFFECT_PRESERVED = 1 << 19;
  USER_EFFECT = 1 << 20;
  WAS_MARKED = 1 << 16;
  REACTION_IS_UPDATING = 1 << 21;
  ASYNC = 1 << 22;
  ERROR_VALUE = 1 << 23;
  STATE_SYMBOL = Symbol("$state");
  LEGACY_PROPS = Symbol("legacy props");
  STALE_REACTION = new class StaleReactionError extends Error {
    name = "StaleReactionError";
    message = "The reaction that called `getAbortSignal()` was re-run or destroyed";
  }();
  COMMENT_NODE = 8;
  HYDRATION_START = "[";
  HYDRATION_START_ELSE = "[!";
  HYDRATION_START_FAILED = "[?";
  HYDRATION_END = "]";
  HYDRATION_ERROR = {};
  const ELEMENT_IS_NAMESPACED = 1;
  const ELEMENT_PRESERVE_ATTRIBUTE_CASE = 1 << 1;
  const ELEMENT_IS_INPUT = 1 << 2;
  UNINITIALIZED = Symbol();
  const DOM_BOOLEAN_ATTRIBUTES = [
    "allowfullscreen",
    "async",
    "autofocus",
    "autoplay",
    "checked",
    "controls",
    "default",
    "disabled",
    "formnovalidate",
    "indeterminate",
    "inert",
    "ismap",
    "loop",
    "multiple",
    "muted",
    "nomodule",
    "novalidate",
    "open",
    "playsinline",
    "readonly",
    "required",
    "reversed",
    "seamless",
    "selected",
    "webkitdirectory",
    "defer",
    "disablepictureinpicture",
    "disableremoteplayback"
  ];
  function is_boolean_attribute(name) {
    return DOM_BOOLEAN_ATTRIBUTES.includes(name);
  }
  const PASSIVE_EVENTS = [
    "touchstart",
    "touchmove"
  ];
  is_passive_event = function(name) {
    return PASSIVE_EVENTS.includes(name);
  };
  const ATTR_REGEX = /[&"<]/g;
  const CONTENT_REGEX = /[&<]/g;
  escape_html = function(value, is_attr) {
    const str = String(value ?? "");
    const pattern = is_attr ? ATTR_REGEX : CONTENT_REGEX;
    pattern.lastIndex = 0;
    let escaped = "";
    let last = 0;
    while (pattern.test(str)) {
      const i = pattern.lastIndex - 1;
      const ch = str[i];
      escaped += str.substring(last, i) + (ch === "&" ? "&amp;" : ch === '"' ? "&quot;" : "&lt;");
      last = i + 1;
    }
    return escaped + str.substring(last);
  };
  const replacements = {
    translate: /* @__PURE__ */ new Map([
      [
        true,
        "yes"
      ],
      [
        false,
        "no"
      ]
    ])
  };
  attr = function(name, value, is_boolean = false) {
    if (name === "hidden" && value !== "until-found") {
      is_boolean = true;
    }
    if (value == null || !value && is_boolean) return "";
    const normalized = has_own_property.call(replacements, name) && replacements[name].get(value) || value;
    const assignment = is_boolean ? `=""` : `="${escape_html(normalized, true)}"`;
    return ` ${name}${assignment}`;
  };
  function clsx(value) {
    if (typeof value === "object") {
      return clsx$1(value);
    } else {
      return value ?? "";
    }
  }
  const whitespace = [
    ..." 	\n\r\f\xA0\v\uFEFF"
  ];
  function to_class(value, hash, directives) {
    var classname = value == null ? "" : "" + value;
    if (hash) {
      classname = classname ? classname + " " + hash : hash;
    }
    if (directives) {
      for (var key of Object.keys(directives)) {
        if (directives[key]) {
          classname = classname ? classname + " " + key : key;
        } else if (classname.length) {
          var len = key.length;
          var a = 0;
          while ((a = classname.indexOf(key, a)) >= 0) {
            var b = a + len;
            if ((a === 0 || whitespace.includes(classname[a - 1])) && (b === classname.length || whitespace.includes(classname[b]))) {
              classname = (a === 0 ? "" : classname.substring(0, a)) + classname.substring(b + 1);
            } else {
              a = b;
            }
          }
        }
      }
    }
    return classname === "" ? null : classname;
  }
  function append_styles(styles, important = false) {
    var separator = important ? " !important;" : ";";
    var css = "";
    for (var key of Object.keys(styles)) {
      var value = styles[key];
      if (value != null && value !== "") {
        css += " " + key + ": " + value + separator;
      }
    }
    return css;
  }
  function to_css_name(name) {
    if (name[0] !== "-" || name[1] !== "-") {
      return name.toLowerCase();
    }
    return name;
  }
  function to_style(value, styles) {
    if (styles) {
      var new_style = "";
      var normal_styles;
      var important_styles;
      if (Array.isArray(styles)) {
        normal_styles = styles[0];
        important_styles = styles[1];
      } else {
        normal_styles = styles;
      }
      if (value) {
        value = String(value).replaceAll(/\s*\/\*.*?\*\/\s*/g, "").trim();
        var in_str = false;
        var in_apo = 0;
        var in_comment = false;
        var reserved_names = [];
        if (normal_styles) {
          reserved_names.push(...Object.keys(normal_styles).map(to_css_name));
        }
        if (important_styles) {
          reserved_names.push(...Object.keys(important_styles).map(to_css_name));
        }
        var start_index = 0;
        var name_index = -1;
        const len = value.length;
        for (var i = 0; i < len; i++) {
          var c = value[i];
          if (in_comment) {
            if (c === "/" && value[i - 1] === "*") {
              in_comment = false;
            }
          } else if (in_str) {
            if (in_str === c) {
              in_str = false;
            }
          } else if (c === "/" && value[i + 1] === "*") {
            in_comment = true;
          } else if (c === '"' || c === "'") {
            in_str = c;
          } else if (c === "(") {
            in_apo++;
          } else if (c === ")") {
            in_apo--;
          }
          if (!in_comment && in_str === false && in_apo === 0) {
            if (c === ":" && name_index === -1) {
              name_index = i;
            } else if (c === ";" || i === len - 1) {
              if (name_index !== -1) {
                var name = to_css_name(value.substring(start_index, name_index).trim());
                if (!reserved_names.includes(name)) {
                  if (c !== ";") {
                    i++;
                  }
                  var property = value.substring(start_index, i).trim();
                  new_style += " " + property + ";";
                }
              }
              start_index = i + 1;
              name_index = -1;
            }
          }
        }
      }
      if (normal_styles) {
        new_style += append_styles(normal_styles);
      }
      if (important_styles) {
        new_style += append_styles(important_styles, true);
      }
      new_style = new_style.trim();
      return new_style === "" ? null : new_style;
    }
    return value == null ? null : String(value);
  }
  const BLOCK_OPEN = `<!--${HYDRATION_START}-->`;
  const BLOCK_CLOSE = `<!--${HYDRATION_END}-->`;
  const EMPTY_COMMENT = `<!---->`;
  let controller = null;
  function abort() {
    controller?.abort(STALE_REACTION);
    controller = null;
  }
  function await_invalid() {
    const error = new Error(`await_invalid
Encountered asynchronous work while rendering synchronously.
https://svelte.dev/e/await_invalid`);
    error.name = "Svelte error";
    throw error;
  }
  function invalid_csp() {
    const error = new Error(`invalid_csp
\`csp.nonce\` was set while \`csp.hash\` was \`true\`. These options cannot be used simultaneously.
https://svelte.dev/e/invalid_csp`);
    error.name = "Svelte error";
    throw error;
  }
  function server_context_required() {
    const error = new Error(`server_context_required
Could not resolve \`render\` context.
https://svelte.dev/e/server_context_required`);
    error.name = "Svelte error";
    throw error;
  }
  function unresolved_hydratable(key, stack) {
    {
      console.warn(`https://svelte.dev/e/unresolved_hydratable`);
    }
  }
  function get_render_context() {
    const store = als?.getStore();
    {
      server_context_required();
    }
    return store;
  }
  let als = null;
  let text_encoder;
  let crypto;
  const obfuscated_import = (module_name) => import(module_name).then(async (m) => {
    await m.__tla;
    return m;
  });
  async function sha256(data) {
    text_encoder ??= new TextEncoder();
    crypto ??= globalThis.crypto?.subtle?.digest ? globalThis.crypto : (await obfuscated_import("node:crypto")).webcrypto;
    const hash_buffer = await crypto.subtle.digest("SHA-256", text_encoder.encode(data));
    return base64_encode(hash_buffer);
  }
  function base64_encode(bytes) {
    if (globalThis.Buffer) {
      return globalThis.Buffer.from(bytes).toString("base64");
    }
    let binary = "";
    for (let i = 0; i < bytes.length; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
  }
  class Renderer {
    #out = [];
    #on_destroy = void 0;
    #is_component_body = false;
    #boundary = null;
    type;
    #parent;
    promise = void 0;
    global;
    local;
    constructor(global, parent) {
      this.#parent = parent;
      this.global = global;
      this.local = parent ? {
        ...parent.local
      } : {
        select_value: void 0
      };
      this.type = parent ? parent.type : "body";
    }
    head(fn) {
      const head2 = new Renderer(this.global, this);
      head2.type = "head";
      this.#out.push(head2);
      head2.child(fn);
    }
    async_block(blockers, fn) {
      this.#out.push(BLOCK_OPEN);
      this.async(blockers, fn);
      this.#out.push(BLOCK_CLOSE);
    }
    async(blockers, fn) {
      let callback = fn;
      if (blockers.length > 0) {
        const context = ssr_context;
        callback = (renderer) => {
          return Promise.all(blockers).then(() => {
            const previous_context = ssr_context;
            try {
              set_ssr_context(context);
              return fn(renderer);
            } finally {
              set_ssr_context(previous_context);
            }
          });
        };
      }
      this.child(callback);
    }
    run(thunks) {
      const context = ssr_context;
      let promise = Promise.resolve(thunks[0]());
      const promises = [
        promise
      ];
      for (const fn of thunks.slice(1)) {
        promise = promise.then(() => {
          const previous_context = ssr_context;
          set_ssr_context(context);
          try {
            return fn();
          } finally {
            set_ssr_context(previous_context);
          }
        });
        promises.push(promise);
      }
      promise.catch(noop);
      this.promise = promise;
      return promises;
    }
    child_block(fn) {
      this.#out.push(BLOCK_OPEN);
      this.child(fn);
      this.#out.push(BLOCK_CLOSE);
    }
    child(fn) {
      const child = new Renderer(this.global, this);
      this.#out.push(child);
      const parent = ssr_context;
      set_ssr_context({
        ...ssr_context,
        p: parent,
        c: null,
        r: child
      });
      const result = fn(child);
      set_ssr_context(parent);
      if (result instanceof Promise) {
        result.catch(noop);
        result.finally(() => set_ssr_context(null)).catch(noop);
        if (child.global.mode === "sync") {
          await_invalid();
        }
        child.promise = result;
      }
      return child;
    }
    boundary(props, children_fn) {
      const child = new Renderer(this.global, this);
      this.#out.push(child);
      const parent_context = ssr_context;
      if (props.failed) {
        child.#boundary = {
          failed: props.failed,
          transformError: this.global.transformError,
          context: parent_context
        };
      }
      set_ssr_context({
        ...ssr_context,
        p: parent_context,
        c: null,
        r: child
      });
      try {
        const result = children_fn(child);
        set_ssr_context(parent_context);
        if (result instanceof Promise) {
          if (child.global.mode === "sync") {
            await_invalid();
          }
          result.catch(noop);
          child.promise = result;
        }
      } catch (error) {
        set_ssr_context(parent_context);
        const failed_snippet = props.failed;
        if (!failed_snippet) throw error;
        const result = this.global.transformError(error);
        child.#out.length = 0;
        child.#boundary = null;
        if (result instanceof Promise) {
          if (this.global.mode === "sync") {
            await_invalid();
          }
          child.promise = result.then((transformed) => {
            child.#out.push(`<!--${HYDRATION_START_FAILED}${JSON.stringify(transformed)}-->`);
            failed_snippet(child, transformed, noop);
            child.#out.push(BLOCK_CLOSE);
          });
          child.promise.catch(noop);
        } else {
          child.#out.push(`<!--${HYDRATION_START_FAILED}${JSON.stringify(result)}-->`);
          failed_snippet(child, result, noop);
          child.#out.push(BLOCK_CLOSE);
        }
      }
    }
    component(fn, component_fn) {
      push();
      const child = this.child(fn);
      child.#is_component_body = true;
      pop();
    }
    select(attrs, fn, css_hash, classes, styles, flags, is_rich) {
      const { value, ...select_attrs } = attrs;
      this.push(`<select${attributes(select_attrs, css_hash, classes, styles, flags)}>`);
      this.child((renderer) => {
        renderer.local.select_value = value;
        fn(renderer);
      });
      this.push(`${is_rich ? "<!>" : ""}</select>`);
    }
    option(attrs, body, css_hash, classes, styles, flags, is_rich) {
      this.#out.push(`<option${attributes(attrs, css_hash, classes, styles, flags)}`);
      const close = (renderer, value, { head: head2, body: body2 }) => {
        if (has_own_property.call(attrs, "value")) {
          value = attrs.value;
        }
        if (value === this.local.select_value) {
          renderer.#out.push(' selected=""');
        }
        renderer.#out.push(`>${body2}${is_rich ? "<!>" : ""}</option>`);
        if (head2) {
          renderer.head((child) => child.push(head2));
        }
      };
      if (typeof body === "function") {
        this.child((renderer) => {
          const r = new Renderer(this.global, this);
          body(r);
          if (this.global.mode === "async") {
            return r.#collect_content_async().then((content) => {
              close(renderer, content.body.replaceAll("<!---->", ""), content);
            });
          } else {
            const content = r.#collect_content();
            close(renderer, content.body.replaceAll("<!---->", ""), content);
          }
        });
      } else {
        close(this, body, {
          body: escape_html(body)
        });
      }
    }
    title(fn) {
      const path = this.get_path();
      const close = (head2) => {
        this.global.set_title(head2, path);
      };
      this.child((renderer) => {
        const r = new Renderer(renderer.global, renderer);
        fn(r);
        if (renderer.global.mode === "async") {
          return r.#collect_content_async().then((content) => {
            close(content.head);
          });
        } else {
          const content = r.#collect_content();
          close(content.head);
        }
      });
    }
    push(content) {
      if (typeof content === "function") {
        this.child(async (renderer) => renderer.push(await content()));
      } else {
        this.#out.push(content);
      }
    }
    on_destroy(fn) {
      (this.#on_destroy ??= []).push(fn);
    }
    get_path() {
      return this.#parent ? [
        ...this.#parent.get_path(),
        this.#parent.#out.indexOf(this)
      ] : [];
    }
    copy() {
      const copy = new Renderer(this.global, this.#parent);
      copy.#out = this.#out.map((item) => item instanceof Renderer ? item.copy() : item);
      copy.promise = this.promise;
      return copy;
    }
    subsume(other) {
      if (this.global.mode !== other.global.mode) {
        throw new Error("invariant: A renderer cannot switch modes. If you're seeing this, there's a compiler bug. File an issue!");
      }
      this.local = other.local;
      this.#out = other.#out.map((item) => {
        if (item instanceof Renderer) {
          item.subsume(item);
        }
        return item;
      });
      this.promise = other.promise;
      this.type = other.type;
    }
    get length() {
      return this.#out.length;
    }
    static render(component, options = {}) {
      let sync;
      const result = {};
      Object.defineProperties(result, {
        html: {
          get: () => {
            return (sync ??= Renderer.#render(component, options)).body;
          }
        },
        head: {
          get: () => {
            return (sync ??= Renderer.#render(component, options)).head;
          }
        },
        body: {
          get: () => {
            return (sync ??= Renderer.#render(component, options)).body;
          }
        },
        hashes: {
          value: {
            script: ""
          }
        },
        then: {
          value: (onfulfilled, onrejected) => {
            {
              const result2 = sync ??= Renderer.#render(component, options);
              const user_result = onfulfilled({
                head: result2.head,
                body: result2.body,
                html: result2.body,
                hashes: {
                  script: []
                }
              });
              return Promise.resolve(user_result);
            }
          }
        }
      });
      return result;
    }
    *#collect_on_destroy() {
      for (const component of this.#traverse_components()) {
        yield* component.#collect_ondestroy();
      }
    }
    *#traverse_components() {
      for (const child of this.#out) {
        if (typeof child !== "string") {
          yield* child.#traverse_components();
        }
      }
      if (this.#is_component_body) {
        yield this;
      }
    }
    *#collect_ondestroy() {
      if (this.#on_destroy) {
        for (const fn of this.#on_destroy) {
          yield fn;
        }
      }
      for (const child of this.#out) {
        if (child instanceof Renderer && !child.#is_component_body) {
          yield* child.#collect_ondestroy();
        }
      }
    }
    static #render(component, options) {
      var previous_context = ssr_context;
      try {
        const renderer = Renderer.#open_render("sync", component, options);
        const content = renderer.#collect_content();
        return Renderer.#close_render(content, renderer);
      } finally {
        abort();
        set_ssr_context(previous_context);
      }
    }
    static async #render_async(component, options) {
      const previous_context = ssr_context;
      try {
        const renderer = Renderer.#open_render("async", component, options);
        const content = await renderer.#collect_content_async();
        const hydratables = await renderer.#collect_hydratables();
        if (hydratables !== null) {
          content.head = hydratables + content.head;
        }
        return Renderer.#close_render(content, renderer);
      } finally {
        set_ssr_context(previous_context);
        abort();
      }
    }
    #collect_content(content = {
      head: "",
      body: ""
    }) {
      for (const item of this.#out) {
        if (typeof item === "string") {
          content[this.type] += item;
        } else if (item instanceof Renderer) {
          item.#collect_content(content);
        }
      }
      return content;
    }
    async #collect_content_async(content = {
      head: "",
      body: ""
    }) {
      await this.promise;
      for (const item of this.#out) {
        if (typeof item === "string") {
          content[this.type] += item;
        } else if (item instanceof Renderer) {
          if (item.#boundary) {
            const boundary_content = {
              head: "",
              body: ""
            };
            try {
              await item.#collect_content_async(boundary_content);
              content.head += boundary_content.head;
              content.body += boundary_content.body;
            } catch (error) {
              const { context, failed, transformError } = item.#boundary;
              set_ssr_context(context);
              let transformed = await transformError(error);
              const failed_renderer = new Renderer(item.global, item);
              failed_renderer.type = item.type;
              failed_renderer.#out.push(`<!--${HYDRATION_START_FAILED}${JSON.stringify(transformed)}-->`);
              failed(failed_renderer, transformed, noop);
              failed_renderer.#out.push(BLOCK_CLOSE);
              await failed_renderer.#collect_content_async(content);
            }
          } else {
            await item.#collect_content_async(content);
          }
        }
      }
      return content;
    }
    async #collect_hydratables() {
      const ctx = get_render_context().hydratable;
      for (const [_, key] of ctx.unresolved_promises) {
        unresolved_hydratable(key, ctx.lookup.get(key)?.stack ?? "<missing stack trace>");
      }
      for (const comparison of ctx.comparisons) {
        await comparison;
      }
      return await this.#hydratable_block(ctx);
    }
    static #open_render(mode, component, options) {
      var previous_context = ssr_context;
      try {
        const renderer = new Renderer(new SSRState(mode, options.idPrefix ? options.idPrefix + "-" : "", options.csp, options.transformError));
        const context = {
          p: null,
          c: options.context ?? null,
          r: renderer
        };
        set_ssr_context(context);
        renderer.push(BLOCK_OPEN);
        component(renderer, options.props ?? {});
        renderer.push(BLOCK_CLOSE);
        return renderer;
      } finally {
        set_ssr_context(previous_context);
      }
    }
    static #close_render(content, renderer) {
      for (const cleanup of renderer.#collect_on_destroy()) {
        cleanup();
      }
      let head2 = content.head + renderer.global.get_title();
      let body = content.body;
      for (const { hash, code } of renderer.global.css) {
        head2 += `<style id="${hash}">${code}</style>`;
      }
      return {
        head: head2,
        body,
        hashes: {
          script: renderer.global.csp.script_hashes
        }
      };
    }
    async #hydratable_block(ctx) {
      if (ctx.lookup.size === 0) {
        return null;
      }
      let entries = [];
      let has_promises = false;
      for (const [k, v] of ctx.lookup) {
        if (v.promises) {
          has_promises = true;
          for (const p of v.promises) await p;
        }
        entries.push(`[${devalue.uneval(k)},${v.serialized}]`);
      }
      let prelude = `const h = (window.__svelte ??= {}).h ??= new Map();`;
      if (has_promises) {
        prelude = `const r = (v) => Promise.resolve(v);
				${prelude}`;
      }
      const body = `
			{
				${prelude}

				for (const [k, v] of [
					${entries.join(",\n					")}
				]) {
					h.set(k, v);
				}
			}
		`;
      let csp_attr = "";
      if (this.global.csp.nonce) {
        csp_attr = ` nonce="${this.global.csp.nonce}"`;
      } else if (this.global.csp.hash) {
        const hash = await sha256(body);
        this.global.csp.script_hashes.push(`sha256-${hash}`);
      }
      return `
		<script${csp_attr}>${body}<\/script>`;
    }
  }
  class SSRState {
    csp;
    mode;
    uid;
    css = /* @__PURE__ */ new Set();
    transformError;
    #title = {
      path: [],
      value: ""
    };
    constructor(mode, id_prefix = "", csp = {
      hash: false
    }, transformError) {
      this.mode = mode;
      this.csp = {
        ...csp,
        script_hashes: []
      };
      this.transformError = transformError ?? ((error) => {
        throw error;
      });
      let uid = 1;
      this.uid = () => `${id_prefix}s${uid++}`;
    }
    get_title() {
      return this.#title.value;
    }
    set_title(value, path) {
      const current = this.#title.path;
      let i = 0;
      let l = Math.min(path.length, current.length);
      while (i < l && path[i] === current[i]) i += 1;
      if (path[i] === void 0) return;
      if (current[i] === void 0 || path[i] > current[i]) {
        this.#title.path = path;
        this.#title.value = value;
      }
    }
  }
  const INVALID_ATTR_NAME_CHAR_REGEX = /[\s'">/=\u{FDD0}-\u{FDEF}\u{FFFE}\u{FFFF}\u{1FFFE}\u{1FFFF}\u{2FFFE}\u{2FFFF}\u{3FFFE}\u{3FFFF}\u{4FFFE}\u{4FFFF}\u{5FFFE}\u{5FFFF}\u{6FFFE}\u{6FFFF}\u{7FFFE}\u{7FFFF}\u{8FFFE}\u{8FFFF}\u{9FFFE}\u{9FFFF}\u{AFFFE}\u{AFFFF}\u{BFFFE}\u{BFFFF}\u{CFFFE}\u{CFFFF}\u{DFFFE}\u{DFFFF}\u{EFFFE}\u{EFFFF}\u{FFFFE}\u{FFFFF}\u{10FFFE}\u{10FFFF}]/u;
  render = function(component, options = {}) {
    if (options.csp?.hash && options.csp.nonce) {
      invalid_csp();
    }
    return Renderer.render(component, options);
  };
  head = function(hash, renderer, fn) {
    renderer.head((renderer2) => {
      renderer2.push(`<!--${hash}-->`);
      renderer2.child(fn);
      renderer2.push(EMPTY_COMMENT);
    });
  };
  function attributes(attrs, css_hash, classes, styles, flags = 0) {
    if (styles) {
      attrs.style = to_style(attrs.style, styles);
    }
    if (attrs.class) {
      attrs.class = clsx(attrs.class);
    }
    if (css_hash || classes) {
      attrs.class = to_class(attrs.class, css_hash, classes);
    }
    let attr_str = "";
    let name;
    const is_html = (flags & ELEMENT_IS_NAMESPACED) === 0;
    const lowercase = (flags & ELEMENT_PRESERVE_ATTRIBUTE_CASE) === 0;
    const is_input = (flags & ELEMENT_IS_INPUT) !== 0;
    for (name of Object.keys(attrs)) {
      if (typeof attrs[name] === "function") continue;
      if (name[0] === "$" && name[1] === "$") continue;
      if (INVALID_ATTR_NAME_CHAR_REGEX.test(name)) continue;
      var value = attrs[name];
      if (lowercase) {
        name = name.toLowerCase();
      }
      if (name.length > 2 && name.startsWith("on")) continue;
      if (is_input) {
        if (name === "defaultvalue" || name === "defaultchecked") {
          name = name === "defaultvalue" ? "value" : "checked";
          if (attrs[name]) continue;
        }
      }
      attr_str += attr(name, value, is_html && is_boolean_attribute(name));
    }
    return attr_str;
  }
  stringify = function(value) {
    return typeof value === "string" ? value : value == null ? "" : value + "";
  };
  attr_class = function(value, hash, directives) {
    var result = to_class(value, hash, directives);
    return result ? ` class="${escape_html(result, true)}"` : "";
  };
  attr_style = function(value, directives) {
    var result = to_style(value, directives);
    return result ? ` style="${escape_html(result, true)}"` : "";
  };
  ensure_array_like = function(array_like_or_iterator) {
    if (array_like_or_iterator) {
      return array_like_or_iterator.length !== void 0 ? array_like_or_iterator : Array.from(array_like_or_iterator);
    }
    return [];
  };
  function once(get_value) {
    let value = UNINITIALIZED;
    return () => {
      if (value === UNINITIALIZED) {
        value = get_value();
      }
      return value;
    };
  }
  derived = function(fn) {
    const get_value = ssr_context === null ? fn : once(fn);
    let updated_value;
    return function(new_value) {
      if (arguments.length === 0) {
        return updated_value ?? get_value();
      }
      updated_value = new_value;
      return updated_value;
    };
  };
})();
export {
  attr_style as $,
  ASYNC as A,
  BOUNDARY_EFFECT as B,
  COMMENT_NODE as C,
  DIRTY as D,
  ERROR_VALUE as E,
  get_descriptor as F,
  get_prototype_of as G,
  HYDRATION_ERROR as H,
  INERT as I,
  is_array as J,
  is_extensible as K,
  USER_EFFECT as L,
  MAYBE_DIRTY as M,
  REACTION_IS_UPDATING as N,
  index_of as O,
  define_property as P,
  array_from as Q,
  REACTION_RAN as R,
  STALE_REACTION as S,
  is_passive_event as T,
  UNINITIALIZED as U,
  LEGACY_PROPS as V,
  WAS_MARKED as W,
  render as X,
  derived as Y,
  ensure_array_like as Z,
  attr_class as _,
  __tla,
  HYDRATION_END as a,
  stringify as a0,
  attr as a1,
  head as a2,
  HYDRATION_START as b,
  HYDRATION_START_ELSE as c,
  EFFECT as d,
  escape_html as e,
  CONNECTED as f,
  CLEAN as g,
  DERIVED as h,
  BLOCK_EFFECT as i,
  deferred as j,
  BRANCH_EFFECT as k,
  ROOT_EFFECT as l,
  RENDER_EFFECT as m,
  noop as n,
  MANAGED_EFFECT as o,
  HEAD_EFFECT as p,
  DESTROYED as q,
  run_all as r,
  includes as s,
  HYDRATION_START_FAILED as t,
  EFFECT_TRANSPARENT as u,
  EFFECT_PRESERVED as v,
  EAGER_EFFECT as w,
  STATE_SYMBOL as x,
  object_prototype as y,
  array_prototype as z
};
