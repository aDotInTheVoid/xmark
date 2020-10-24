# xMark

## Prior Art

- [mdBook](https://github.com/rust-lang/mdbook)
- [GitHub Docs](https://github.com/github/docs)
- [fasterthanlime's new site](https://fasterthanli.me/articles/a-new-website-for-2020)
- [tavianator.com](https://github.com/tavianator/tavianator.com)
- [tumblelog](https://github.com/john-bokma/tumblelog)
- jekyll
- hugo
- asciidoctor
- ninja
- https://joshwcomeau.com/css/full-bleed/
- doctave

## Drawing forms

- [penrose](https://github.com/penrose/penrose)
- [graphviz](https://graphviz.org/)
- [ditaa](https://github.com/stathissideris/ditaa)
- [plantUML](https://plantuml.com/)
- [draw.io](https://app.diagrams.net/)?
- tikz?
- [libfsm](https://github.com/katef/libfsm)?
- [kgt](https://github.com/katef/kgt)?
- metapost?
- gnuplot?
- pyplot?
- Any of the other formats suported by [adoc](https://github.com/asciidoctor/asciidoctor-diagram/blob/fd8ab7d9eb9d5de3c55a0e27c4276206c728a917/README.adoc#creating-a-diagram)

## Outputs

- HTML
- PDF?

## Inputs

- Markdown
- Asciidoc?


```css
.wrapper {
  display: grid;
  grid-template-columns:
    1fr
    min(65ch, 100%)
    1fr;
}
.wrapper > * {
  grid-column: 2;
}
.full-bleed {
  width: 100%;
  grid-column: 1 / 4;
}
```
