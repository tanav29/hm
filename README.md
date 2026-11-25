# twhtml

A compact syntax for generating HTML with Tailwind CSS. By default, twhtml outputs a complete HTML document using the Tailwind browser CDN.

```text
main.min-h-screen {
  img#avatar%photo.jpg&"Portrait of Alex"
  a.text-blue-500$/about "About"
}
```

## Usage

Install Rust, then run:

```sh
cargo run -- examples/landing.hm -o landing.html
cargo run -- --fragment examples/landing.hm
```

Omit the input path (or use `-`) to read stdin. Omit `-o` to save an HTML file named after the input in the OS temporary directory (or `output.html` for stdin). `--fragment` omits the document wrapper and CDN. The default document uses English and the title `Document`; use a fragment for a custom wrapper. The CDN requires network access and is intended for prototypes; use a Tailwind build for production.

The CLI reports input and output character counts, parse and render time, and the output location on stderr. Add `--tokens` to also report estimated `o200k_base` token savings, with and without the skill file. Token counting adds overhead; counts vary by model, and the skill should be counted once when using multiple files.

## Syntax

| Shorthand | Meaning | Example |
| --- | --- | --- |
| `.class` | CSS or Tailwind class | `div.flex.md:flex` |
| `#id` | Element ID | `div#main` |
| `$value` | `href` attribute | `a$/about "About"` |
| `%value` | `src` attribute | `img%photo.jpg` |
| `&value` | `alt` attribute | `img&"A portrait"` |

Put selectors before attribute shorthands. Quote shorthand values containing whitespace, `#`, `%`, `&`, `$`, brackets, or braces. Other attributes use `[name="value"]` and can be repeated; IDs may also use `[id="value"]`.

Put quoted text after a space and children inside `{ ... }`. Comments start with `//` between elements. Strings support `\"`, `\\`, `\n`, and `\t`; text and attribute values are HTML-escaped. Void elements such as `img` and `input` cannot contain text or children.

Tailwind variants and arbitrary values work, for example `.hover:bg-blue-500`, `.w-[2.5rem]`, and `.[&>p]:text-red-500`. Use underscores for spaces inside brackets; decimal class names outside brackets are unsupported. Errors include a line and column.
