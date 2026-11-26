---
name: hol-markup
description: Write compact .hm files for the hm HTML and Tailwind transpiler.
---

# HOL markup

Use `tag.class#id[attr="value"] "text" { children }`. Text and children are optional; interleave quoted text and elements inside a block for mixed content, e.g. `p { "Hello " strong "world" "!" }`; a leading `.class` or `#id` implies `div`. Separate siblings with whitespace.

```text
main.p-6 {
  img%"photo.jpg"&"Portrait"
  a.hover:underline$"/about" "About"
  input! type=email name=address
}
```

- Put selectors before attributes; avoid duplicate IDs or attributes and `[class=...]`.
- `$`, `%`, `&`, `!` set `href`, `src`, `alt`, and `required`. Always double-quote `href` and `src` values. Quote alt shorthand values containing whitespace or `# % & $ ! [ ] { }`.
- Other attributes: ` name=value`, ` name="value with spaces"`, or `[name="value"]`; `[name]` emits an empty value. Unquoted spaced values cannot contain whitespace, brackets, braces, or quotes.
- Quote text. Strings support `\"`, `\\`, `\n`, `\t`; output is HTML-escaped. `//` comments occur between elements. Void elements cannot have text or children.
- Tailwind variants and arbitrary values work: `.md:flex`, `.w-[2.5rem]`, `.[&>p]:text-red-500`. Use underscores for spaces in brackets; decimal classes outside brackets are unsupported.
- Run `cargo run -- input.hm -o output.html`; add `--fragment` to omit the document wrapper and Tailwind CDN, or `--tokens` for token estimates. Omit `-o` to write HTML to stdout. Errors include line and column; nesting is limited to 128 levels.
