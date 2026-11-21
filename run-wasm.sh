cargo build --package calculator-parsers --lib --target wasm32-unknown-unknown --profile release-wasm
wasm-bindgen --target web --out-dir ./webdemo/wasm-pkg ./target/wasm32-unknown-unknown/release-wasm/calculator_parsers.wasm
wasm-opt -Os --strip-debug --strip-producers ./webdemo/wasm-pkg/calculator_parsers_bg.wasm -o ./webdemo/wasm-pkg/calculator_parsers_bg.wasm
stat --printf="wasm size %s\n" ./webdemo/wasm-pkg/calculator_parsers_bg.wasm
cd webdemo
(exec python3 -m http.server 8000) &
SERVER_PID=$!
open http://127.0.0.1:8000
wait "$SERVER_PID"
