---
name: hol-markup
description: Write compact .hm files for the twhtml HTML and Tailwind transpiler.
---

# HOL markup

Write `tag.class#id[attr="value"] "text" { children }`. Text and children are optional. A leading `.class` or `#id` implies `div`. Separate sibling elements with whitespace; use newlines for readability.

```text
main.p-6 {
  .flex.gap-4 {
    img%photo.jpg&"Portrait"
    a.hover:underline$/about "About"
  }
  input[type="email"]!
}
```

- `.class` adds a class, `#id` an ID. Use selectors before attributes. Repeat selectors and attributes as needed; don't use `[class=...]` or duplicate IDs/attributes.
- `$`, `%`, and `&` set `href`, `src`, and `alt`. Quote values containing whitespace or `# % & $ ! [ ] { }` (e.g. `$"/?a=1&b=2"`). Other attributes use `[name="value"]`; `!` sets `required`; `[required]` also works. `[name]` emits an empty value, useful for boolean attributes.
- Quote text. Strings support `\"`, `\\`, `\n`, and `\t`; output is HTML-escaped. `//` starts a comment between elements. Void elements cannot contain text or children.
- Tailwind variants and arbitrary values work: `.md:flex`, `.w-[2.5rem]`, `.[&>p]:text-red-500`. Use underscores for spaces inside brackets; decimal classes outside brackets are unsupported.
- Run `cargo run -- input.hm -o output.html`. Output is a complete document using the Tailwind browser CDN; use `--fragment` to omit the wrapper and CDN. Add `--tokens` for token estimates. Errors report line and column.
