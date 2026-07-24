#!/bin/sh
set -eu

case "${1:-}" in
  --contrapunk) products="contrapunk" ;;
  --elixir) products="elixir" ;;
  --all) products="contrapunk elixir" ;;
  *) echo "usage: $0 --contrapunk|--elixir|--all" >&2; exit 2 ;;
esac

root="${PLUGIN_ROOT:-/}"
for product in $products; do
  case "$product" in
    contrapunk)
      rm -rf \
        "$root/Library/Audio/Plug-Ins/VST3/Contrapunk.vst3" \
        "$root/Library/Audio/Plug-Ins/CLAP/Contrapunk.clap" \
        "$root/Library/Audio/Plug-Ins/Components/Contrapunk.component" \
        "$root/Library/Audio/Plug-Ins/Components/Contrapunk Guitar.component"
      ;;
    elixir)
      rm -rf \
        "$root/Library/Audio/Plug-Ins/VST3/Elixir.vst3" \
        "$root/Library/Audio/Plug-Ins/CLAP/Elixir.clap"
      ;;
  esac
done

if [ "$root" = / ]; then
  /usr/bin/killall -9 AudioComponentRegistrar >/dev/null 2>&1 || true
fi
