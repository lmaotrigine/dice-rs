set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]
python := if os_family() == 'windows' { 'py -3' } else { 'python3' }

_default:
  @just --list

package-name := 'dice'
index-header := (
  '<!DOCTYPE html>'
  + '<html>'
  +   '<head><meta http-equiv="refresh" content="0;url='+ package-name + '/index.html"></head>'
  +   '<body>'
  +     '<p>Redirecting to <a href="./' + package-name + '/index.html">./' + package-name + '/index.html</a>...</p>'
  +   '</body>'
  + '</html>'
)

# generate documentation suitable for self-hosting
[unix]
doc:
  #!/bin/sh
  cargo clean --doc
  cargo +nightly doc --all-features --no-deps
  rm -f target/doc/.lock
  printf '{{index-header}}' > target/doc/index.html

# generate documentation suitable for self-hosting
[windows]
doc:
  #!pwsh
  cargo clean --doc
  cargo +nightly doc --all-features --no-deps
  if (Test-Path target\doc\.lock) { Remove-Item target\doc\.lock }
  Write-Output '{{index-header}}' | Out-File -FilePath target\doc\index.html -Encoding utf8

# run clippy on all features
[env('RUSTFLAGS', '-Wunused-crate-dependencies')]
clippy:
  cargo hack clippy --each-feature

# run tests using cargo-nextest and also doctests
test *args="":
  cargo nextest run --all-features --verbose {{args}}
  cargo test --doc --all-features

# (re)generate README.md from crate level documentation
readme:
  {{python}} generate_readme.py

# check if README.md is up-to-date with crate level documentation
readme-check:
  {{python}} generate_readme.py check

# run cargo fmt
fmt *args="":
  cargo fmt {{args}}

# run all checks
ci: clippy (test "--profile ci") (fmt "-- --check") readme-check

# vim: ts=2 sts=2 sw=2 et
