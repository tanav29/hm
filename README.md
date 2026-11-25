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
| `.class` | CSS or Tailwind class; leading selector implies `div` | `.flex.md:flex` |
| `#id` | Element ID | `div#main` |
| `$value` | `href` attribute | `a$/about "About"` |
| `%value` | `src` attribute | `img%photo.jpg` |
| `&value` | `alt` attribute | `img&"A portrait"` |
| `!` | Required attribute | `input!` |

Put selectors before attribute shorthands. Quote shorthand values containing whitespace, `#`, `%`, `&`, `$`, `!`, brackets, or braces. Other attributes can be written after a space, such as `input.field type=email name=address`, or in brackets as `[name="value"]`. Spaced attribute values can be unquoted when they contain no whitespace, brackets, braces, or quotes; use double quotes otherwise or for an empty value. Bracketed values require double quotes. `[required]` renders an empty value for boolean attributes; IDs may also use `id=value` or `[id="value"]`.

Put quoted text after a space and children inside `{ ... }`. Comments start with `//` between elements. Strings support `\"`, `\\`, `\n`, and `\t`; text and attribute values are HTML-escaped. Void elements such as `img` and `input` cannot contain text or children.

Tailwind variants and arbitrary values work, for example `.hover:bg-blue-500`, `.w-[2.5rem]`, and `.[&>p]:text-red-500`. Use underscores for spaces inside brackets; decimal class names outside brackets are unsupported. Errors include a line and column.

## Visual example

[`examples/stillwater.hm`](examples/stillwater.hm) is a coastal travel journal with responsive layouts, nested elements, arbitrary Tailwind values, image and link shorthand, and escaped URLs. Its generated artifact is [`examples/stillwater.html`](examples/stillwater.html). The page loads its photographs and Tailwind from external services.

```sh
cargo run --release -- examples/stillwater.hm -o examples/stillwater.html --tokens
```

One release-mode run produced 4,235 input chars - 1,267 input tokens and 5,259 output chars - 1,570 output tokens in 0.167 ms (parse and render only). The `o200k_base` estimate was 1,267 tokens for the source, 1,570 for the HTML, and 398 for the current skill: a saving of 303 tokens (19.3%), or -95 (-6.1%) including the skill once. Timing varies by machine and run; token counts depend on the tokenizer.

Two shorter examples are also available:

| Example | Source characters | HTML characters | Source tokens | HTML tokens | Token savings |
| --- | ---: | ---: | ---: | ---: | ---: |
| [Morrow café](examples/morrow.hm) ([HTML](examples/morrow.html)) | 2,236 | 3,005 | 696 | 926 | 230 (24.8%) |
| [Recipe](examples/recipe.hm) ([HTML](examples/recipe.html)) | 1,279 | 1,712 | 380 | 528 | 148 (28.0%) |

These token savings exclude the 398-token skill; including it once gives -168 (-18.1%) and -250 (-47.3%), respectively. The examples use external photographs and the Tailwind CDN.

[`examples/compact.hm`](examples/compact.hm) demonstrates implicit `div` selectors and a boolean attribute in 468 characters ([HTML](examples/compact.html)). One run estimated 137 source tokens and 250 HTML tokens: 45.2% fewer without the skill, or 114.0% more including the skill once.
