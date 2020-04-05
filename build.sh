#!/bin/bash

rm -r dist
rm -r pkg

set -e

wasm-pack build -t no-modules
./node_modules/.bin/webpack --config web/webpack.config.js --mode production

cd dist

cat ../index.html | \
  sed -e "s;web/style.css;$(ls -1 *css);" \
      -e "s;main.js;$(ls -1 index*js);" \
  > index.html