---
name: hol-markup
description: Write .hm for the twhtml HTML and Tailwind transpiler.
---

# HOL markup

Write elements as `tag.class#id[attr="value"] "text" { children }` (text and children optional). Repeat attributes; use `.class`, not `[class=...]`. Example:

```text
main.min-h-screen {
  img%photo.jpg&"Portrait"
  a.hover:underline$/about "About"
}
```

`$` sets href, `%` src, `&` alt. Put selectors first; quote shorthand values containing whitespace, `#`, `%`, `&`, `$`, brackets or braces. `//` starts a comment between elements. Strings support `\"`, `\\`, `\n`, `\t`; output is HTML-escaped. Void elements cannot have content.

Tailwind variants and arbitrary values work, e.g. `.md:flex`, `.w-[2.5rem]`, `.[&>p]:text-red-500`; use underscores for spaces in brackets. Decimal classes outside brackets are unsupported.

Run `cargo run -- input.hm -o output.html`. Default output is a complete document with Tailwind browser CDN (for prototypes); `--fragment` omits the wrapper and CDN.
