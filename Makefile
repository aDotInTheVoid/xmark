.PHONY: fmt
fmt: 
	cargo fmt
	./node_modules/.bin/js-beautify -r www/templates/*.html

.PHONY: cy-open
cy-open:
	./node_modules/.bin/cypress open

.PHONY: cy-test
cy-test:
	./node_modules/.bin/cypress run --headless

.PHONY: serve-dummy
serve-dummy:
	cargo run -- -d dummy-book/
	python3 -m http.server -d dummy-book/_out/html 9009

.PHONY: docs
docs:
	cargo doc --document-private-items --no-deps

docs-serve: docs
	python3 -m http.server -d target/doc

.PHONY: docs-json
docs-json:
	cargo rustdoc -- --output-format json 
