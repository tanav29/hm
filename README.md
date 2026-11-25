# twhtml

A compact, agent-friendly syntax for generating HTML with Tailwind CSS. The CLI produces a complete HTML document using the Tailwind browser CDN by default.

```text
div.bg-red-300.rounded-md {
  p.text-md "hello"
}
```

## Usage

Install Rust, then run:

```sh
cargo run -- examples/landing.hm -o landing.html
cargo run -- --fragment examples/landing.hm
```

After conversion, the CLI reports estimated token savings on stderr using the `o200k_base` tokenizer. It compares the generated HTML with the source, both alone and including the skill file once. Counts vary by model; for multiple files, count the skill only once. HTML on stdout stays clean.

Omit the input path (or use `-`) to read standard input; omit `-o` to write to standard output. `--fragment` emits only the elements, without a document wrapper or Tailwind script.

## Syntax

- Start an element with its tag, followed immediately by `.class` selectors.
- Add `$URL` after a tag or its classes as shorthand for `href`, for example `a.text-blue-500$/about "About"`. Use `$"URL"` for URLs containing whitespace, brackets, or braces; put classes before the shorthand.
- Add attributes immediately after the selectors as `[name="value"]`; repeat for multiple attributes. Use `.class` and `#id` selectors, or `[id="value"]` for IDs.
- Add quoted text after a space, and nest elements inside `{ ... }`. Comments start with `//` between elements.
- Strings support `\"`, `\\`, `\n`, and `\t`. Text and attribute values are HTML-escaped.
- Responsive and state classes such as `.md:flex` and `.hover:bg-blue-500` work. Arbitrary values and variants work, for example `.w-[2.5rem]`, `.bg-[#123456]`, and `.[&>p]:text-red-500`. Use underscores in place of spaces inside brackets. Decimal class names outside brackets are not supported.
- Void elements such as `img` and `input` cannot have children or text. Errors include a line and column.

The browser CDN is convenient for prototypes and requires network access; Tailwind recommends a build step for production. The generated wrapper has a default title and English language setting; use `--fragment` to integrate into a custom document.


Use `%value` for `src` and `&value` for `alt`, for example `img%photo.jpg&"Portrait of Alex"`. Use quoted values when they contain whitespace, `#`, `%`, `&`, `$`, brackets, or braces. `$value` sets `href`.
