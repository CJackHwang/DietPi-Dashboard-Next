#!/bin/bash -ex

asset_path='crates/frontend/assets'
dist_path='crates/frontend/dist'

js_assets=(
  "$asset_path/js/xterm-5.5.0.js"
  "$asset_path/js/xterm-addon-fit-0.10.0.js"
  "$asset_path/js/microlight-0.0.7.js"
  "$asset_path/js/nomini-0.3.0-custom.js"
  "$asset_path/js/components.js"
)

css_assets=(
  "$asset_path/css/vars-clean.css"
  "$asset_path/css/global.css"
  "$asset_path/css/system.css"
  "$asset_path/css/process.css"
  "$asset_path/css/management.css"
  "$asset_path/css/software.css"
  "$asset_path/css/browser.css"
  "$asset_path/css/xterm-5.5.0.css"
)

svg_assets=(
  "$asset_path/icons.svg"
  "$asset_path/favicon.svg"
)

js_out="$dist_path/main.js"
css_out="$dist_path/main.css"

mkdir -p "$dist_path"

./scripts/clean-css.bash "${css_assets[@]:1}" > "${css_assets[0]}"

minify -b "${js_assets[@]}" | brotli -fo "$js_out"
minify -b "${css_assets[@]}" | brotli -fo "$css_out"

for svg in "${svg_assets[@]}"; do
  brotli "$svg" -fo "$dist_path/$(basename "$svg")"
done

rm "${css_assets[0]}"
