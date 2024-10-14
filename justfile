default: watch

build_css:
  npm run build
build: build_css
  cargo build --release
watch:
  cargo-watch -x check  -s 'just build_css && cargo loco start'
