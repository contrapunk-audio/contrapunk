import "clsx";
function _layout($$renderer, $$props) {
  let { children } = $$props;
  $$renderer.push(`<div class="diary-shell svelte-1mi82qd">`);
  children($$renderer);
  $$renderer.push(`<!----></div>`);
}
export {
  _layout as default
};
