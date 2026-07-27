default: watch

build:
  cargo build --release
watch:
  cargo-watch -x check  -s 'cargo loco start'
