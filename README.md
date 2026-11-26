# hm

A compact syntax for generating HTML with Tailwind CSS. By default, hm outputs a complete HTML document using the Tailwind browser CDN.

```text
main.min-h-screen {
  img#avatar%"photo.jpg"&"Portrait of Alex"
  a.text-blue-500$"/about" "About"
}
```

## Usage

Install Rust, then run:

```sh
cargo run -- examples/landing.hm -o landing.html
cargo run -- --fragment examples/landing.hm
```

Omit the input path (or use `-`) to read stdin. Omit `-o` or use `-o -` to write HTML to stdout; diagnostics go to stderr. An explicit output path overwrites that file. `--fragment` omits the document wrapper and CDN. The default document uses English and the title `Document`; set a custom title with `--title "My page"`. The title has no effect on fragments. The CDN requires network access and is intended for prototypes; use a Tailwind build for production.

Alternatively, put `title "My page"` on the first non-comment line of the `.hm` file. The declaration is omitted from fragment output. If both are present, `--title` takes precedence.

The CLI reports input and output character counts, parse and render time, and the output location on stderr. Add `--tokens` to also report estimated `o200k_base` token savings, with and without the skill file. Token counting adds overhead; counts vary by model, and the skill should be counted once when using multiple files.

## Syntax

| Shorthand | Meaning | Example |
| --- | --- | --- |
| `.class` | CSS or Tailwind class; leading selector implies `div` | `.flex.md:flex` |
| `#id` | Element ID | `div#main` |
| `$value` | `href` attribute | `a$"/about" "About"` |
| `%value` | `src` attribute | `img%"photo.jpg"` |
| `&value` | `alt` attribute | `img&"A portrait"` |
| `!` | Required attribute | `input!` |

Put selectors before attribute shorthands. Always double-quote URL values for `href` and `src`, including spaced attributes. Quote alt shorthand values containing whitespace, `#`, `%`, `&`, `$`, `!`, brackets, or braces. Other attributes can be written after a space, such as `input.field type=email name=address`, or in brackets as `[name="value"]`. Spaced attribute values can be unquoted when they contain no whitespace, brackets, braces, or quotes; use double quotes otherwise or for an empty value. Bracketed values require double quotes. `[required]` renders an empty value for boolean attributes; IDs may also use `id=value` or `[id="value"]`.

Put quoted text after a space and children inside `{ ... }`. For mixed content, interleave quoted text and elements inside the block, for example `p { "Hello " strong "world" "!" }`. Nesting is limited to 128 levels. Comments start with `//` between elements. Strings support `\"`, `\\`, `\n`, and `\t`; text and attribute values are HTML-escaped. Void elements such as `img` and `input` cannot contain text or children.

Tailwind variants and arbitrary values work, for example `.hover:bg-blue-500`, `.w-[2.5rem]`, and `.[&>p]:text-red-500`. Use underscores for spaces inside brackets; decimal class names outside brackets are unsupported. Errors include a line and column.

## Visual example

[`examples/stillwater.hm`](examples/stillwater.hm) is a coastal travel journal with responsive layouts, nested elements, arbitrary Tailwind values, image and link shorthand, and escaped URLs. Its generated artifact is [`examples/stillwater.html`](examples/stillwater.html). The page loads its photographs and Tailwind from external services.

```sh
cargo run --release -- examples/stillwater.hm -o examples/stillwater.html --tokens
```

The `--tokens` option compares the source and generated HTML using `o200k_base`, and reports the skill overhead separately. For multiple files, add the source token counts and count the skill once; compare that total with the sum of the HTML counts. Savings depend on the content and model tokenizer. These examples do not establish savings for real multi-page workflows.

The remaining examples illustrate shorter pages and common elements. Generate fresh estimates with `--tokens` when evaluating your own workload.
