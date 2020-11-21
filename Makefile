.PHONY: fmt
fmt: 
	cargo fmt
	./node_modules/.bin/js-beautify -r www/templates/*.html
