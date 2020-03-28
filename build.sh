#!/bin/bash

rm -r dist
rm -r pkg

set -e

wasm-pack build -t no-modules
./node_modules/.bin/webpack --config web/webpack.config.js --mode production

./node_modules/.bin/terser -c -m -- pkg/explainers.js > dist/explainers.js
mv dist/explainers.js dist/explainers-$(md5sum dist/explainers.js | cut -d ' ' -f1).js

cp pkg/explainers_bg.wasm dist/explainers_bg-$(md5sum pkg/explainers_bg.wasm | cut -d ' ' -f1).wasm

cat web/index.js | \
  sed -e 's;self.IS_WORKER;true;' -e 's;self.SKIP_LOGGING;true;' | \
  ./node_modules/.bin/terser -c -m > dist/worker.js

mv dist/worker.js dist/worker-$(md5sum dist/worker.js | cut -d ' ' -f1).js

cd dist

cat ../index.html | \
  sed -e "s;web/style.css;$(ls -1 *css);" \
      -e "s;workerMain *= *\"web/index.js\";workerMain = \"$(ls -1 worker*js)\";" \
      -e "s;web/index.js;$(ls -1 index*js);" \
      -e "s;/pkg/explainers.js;$(ls -1 explainers*js);" \
      -e "s;/pkg/explainers_bg.wasm;$(ls -1 explainers*wasm);" \
  > index.html