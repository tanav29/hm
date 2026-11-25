---
name: hol-markup
description: Write or edit .hm files for the twhtml HTML and Tailwind transpiler.
---

# HOL markup

Write `.hm` files as a sequence of HTML elements. Use `tag.class.class#id`, immediately followed by attributes as `[name="value"]`; put quoted text after a space and children in braces. Tailwind classes are enabled by default.

```text
main.min-h-screen.bg-slate-950.text-white {
  h1.text-4xl.font-bold "Hello"
  a.text-blue-400.hover:underline[href="/about"] "About"
}
```

Use `//` comments between elements. Strings accept `\"`, `\\`, `\n`, and `\t`; the compiler escapes text and attribute values. Void elements cannot contain text or children. Use selectors for class and id, not attributes. Colon variants such as `.md:flex` work; arbitrary values and variants work (e.g. `.w-[2.5rem]`, `.[&>p]:text-red-500`); use underscores for spaces in brackets. Decimal class names outside brackets are not supported.

Compile with `cargo run -- input.hm -o output.html` from the twhtml project. Use `--fragment` to omit the document wrapper and Tailwind browser CDN. The CDN is intended for prototypes; use a Tailwind build for production.
