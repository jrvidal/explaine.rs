set -e

cd playground
wasm-pack build -t web -d ../pkg
cd ..

npx vite build
