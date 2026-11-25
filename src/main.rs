use std::{
    env, fs,
    io::{self, Read, Write},
    process,
    time::Instant,
};

#[derive(Debug)]
struct Node {
    tag: String,
    classes: Vec<String>,
    id: Option<String>,
    attrs: Vec<(String, String)>,
    text: Option<String>,
    children: Vec<Node>,
}
struct Parser {
    chars: Vec<char>,
    pos: usize,
}
impl Parser {
    fn new(s: &str) -> Self {
        Self {
            chars: s.trim_start_matches('\u{feff}').chars().collect(),
            pos: 0,
        }
    }
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }
    fn error(&self, msg: &str) -> String {
        let before: String = self.chars[..self.pos].iter().collect();
        let line = before.chars().filter(|&c| c == '\n').count() + 1;
        let col = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        format!("{line}:{col}: {msg}")
    }
    fn space(&mut self) {
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.bump();
            }
            if self.peek() == Some('/') && self.chars.get(self.pos + 1) == Some(&'/') {
                while self.peek().is_some_and(|c| c != '\n') {
                    self.bump();
                }
            } else {
                break;
            }
        }
    }
    fn word(&mut self) -> String {
        let start = self.pos;
        while self.peek().is_some_and(|c| {
            !c.is_whitespace()
                && !matches!(
                    c,
                    '.' | '#' | '{' | '}' | '[' | ']' | '=' | '"' | '$' | '&' | '%' | '!'
                )
        }) {
            self.bump();
        }
        self.chars[start..self.pos].iter().collect()
    }
    fn class_name(&mut self) -> Result<String, String> {
        let mut value = self.word();
        loop {
            if self.peek() != Some('[')
                || !(value.is_empty() || value.ends_with('-') || value.ends_with(':'))
            {
                break;
            }
            let mut depth = 0usize;
            loop {
                match self.bump() {
                    Some('[') => {
                        depth += 1;
                        value.push('[');
                    }
                    Some(']') => {
                        depth -= 1;
                        value.push(']');
                        if depth == 0 {
                            break;
                        }
                    }
                    Some(c) if !c.is_whitespace() => value.push(c),
                    Some(_) => {
                        return Err(self.error(
                            "whitespace is not allowed in arbitrary values; use underscores",
                        ));
                    }
                    None => return Err(self.error("unterminated arbitrary value")),
                }
            }
            value.push_str(&self.word());
        }
        Ok(value)
    }
    fn quoted(&mut self) -> Result<String, String> {
        if self.bump() != Some('"') {
            return Err(self.error("expected a quoted string"));
        }
        let mut out = String::new();
        loop {
            match self.bump() {
                Some('"') => return Ok(out),
                Some('\\') => match self.bump() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    _ => return Err(self.error("invalid escape")),
                },
                Some(c) => out.push(c),
                None => return Err(self.error("unterminated string")),
            }
        }
    }
    fn nodes(&mut self, nested: bool, depth: usize) -> Result<Vec<Node>, String> {
        if depth > 128 {
            return Err(self.error("maximum nesting depth exceeded"));
        }
        let mut nodes = Vec::new();
        loop {
            self.space();
            match self.peek() {
                None if nested => return Err(self.error("expected '}'")),
                None => break,
                Some('}') if nested => {
                    self.bump();
                    break;
                }
                Some('}') => return Err(self.error("unexpected '}'")),
                Some('"') if nested => nodes.push(Node {
                    tag: String::new(),
                    classes: vec![],
                    id: None,
                    attrs: vec![],
                    text: Some(self.quoted()?),
                    children: vec![],
                }),
                _ => nodes.push(self.node(depth)?),
            }
        }
        Ok(nodes)
    }
    fn node(&mut self, depth: usize) -> Result<Node, String> {
        // A selector without a tag is a compact div, as in CSS and Emmet.
        let tag = if matches!(self.peek(), Some('.') | Some('#')) {
            "div".to_string()
        } else {
            self.word()
        };
        if tag.is_empty()
            || !tag.chars().next().unwrap().is_ascii_alphabetic()
            || !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err(self.error("expected an HTML tag"));
        }
        let mut n = Node {
            tag,
            classes: vec![],
            id: None,
            attrs: vec![],
            text: None,
            children: vec![],
        };
        loop {
            match self.peek() {
                Some('!') => {
                    self.bump();
                    if n.attrs.iter().any(|(k, _)| k == "required") {
                        return Err(self.error("duplicate required"));
                    }
                    n.attrs.push(("required".into(), String::new()));
                }
                Some('.') | Some('#') => {
                    let marker = self.bump().unwrap();
                    let value = if marker == '.' {
                        self.class_name()?
                    } else {
                        self.word()
                    };
                    if value.is_empty() {
                        return Err(self.error("expected a name after selector"));
                    }
                    if marker == '.' {
                        n.classes.push(value);
                    } else if n.attrs.iter().any(|(k, _)| k == "id")
                        || n.id.replace(value).is_some()
                    {
                        return Err(self.error("duplicate id"));
                    }
                }
                Some('$') | Some('%') | Some('&') => {
                    let marker = self.bump().unwrap();
                    let key = match marker {
                        '$' => "href",
                        '%' => "src",
                        _ => "alt",
                    };
                    if marker != '&' && self.peek() != Some('"') {
                        return Err(self.error("URL shorthand values must be double-quoted"));
                    }
                    let url = if self.peek() == Some('"') {
                        self.quoted()?
                    } else {
                        let start = self.pos;
                        while self.peek().is_some_and(|c| {
                            !c.is_whitespace()
                                && !matches!(c, '{' | '}' | '[' | ']' | '&' | '#' | '$' | '%' | '!')
                        }) {
                            self.bump();
                        }
                        self.chars[start..self.pos].iter().collect()
                    };
                    if url.is_empty() {
                        return Err(self.error("expected a value after shorthand"));
                    }
                    if n.attrs.iter().any(|(k, _)| k == key) {
                        return Err(self.error(&format!("duplicate {key}")));
                    }
                    n.attrs.push((key.into(), url));
                }
                Some('[') => {
                    self.bump();
                    self.space();
                    let key = self.word();
                    if key.is_empty()
                        || !key
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':'))
                    {
                        return Err(self.error("invalid attribute name"));
                    }
                    let key = key.to_ascii_lowercase();
                    if !key
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == ':')
                    {
                        return Err(self.error("invalid attribute name"));
                    }
                    if key == "class" {
                        return Err(self.error("use .class selectors"));
                    }
                    self.space();
                    let value = if self.peek() == Some(']') {
                        String::new()
                    } else {
                        if self.bump() != Some('=') {
                            return Err(self.error("expected '=' or ']'"));
                        }
                        self.space();
                        self.quoted()?
                    };
                    self.space();
                    if self.bump() != Some(']') {
                        return Err(self.error("expected ']'"));
                    }
                    if (key == "id" && n.id.is_some()) || n.attrs.iter().any(|(k, _)| k == &key) {
                        return Err(self.error(&format!("duplicate {key}")));
                    }
                    n.attrs.push((key, value));
                }
                _ => break,
            }
        }
        // Horizontal whitespace permits text or a block without consuming the next element.
        while self
            .peek()
            .is_some_and(|c| c == ' ' || c == '\t' || c == '\r')
        {
            self.bump();
        }
        // An equals sign distinguishes a spaced attribute from the next sibling.
        loop {
            let start = self.pos;
            while self
                .peek()
                .is_some_and(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':'))
            {
                self.bump();
            }
            if self.pos == start || self.peek() != Some('=') {
                self.pos = start;
                break;
            }
            let key: String = self.chars[start..self.pos].iter().collect();
            let key = key.to_ascii_lowercase();
            if !key
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == ':')
            {
                return Err(self.error("invalid attribute name"));
            }
            if key == "class" {
                return Err(self.error("use .class selectors"));
            }
            if (key == "id" && n.id.is_some()) || n.attrs.iter().any(|(k, _)| k == &key) {
                return Err(self.error(&format!("duplicate {key}")));
            }
            self.bump();
            if matches!(key.as_str(), "href" | "src") && self.peek() != Some('"') {
                return Err(self.error("URL attribute values must be double-quoted"));
            }
            let value = if self.peek() == Some('"') {
                self.quoted()?
            } else {
                let start = self.pos;
                while self.peek().is_some_and(|c| {
                    !c.is_whitespace() && !matches!(c, '{' | '}' | '[' | ']' | '"' | '\'')
                }) {
                    self.bump();
                }
                if self.pos == start {
                    return Err(self.error("expected an attribute value"));
                }
                self.chars[start..self.pos].iter().collect()
            };
            n.attrs.push((key, value));
            while self
                .peek()
                .is_some_and(|c| c == ' ' || c == '\t' || c == '\r')
            {
                self.bump();
            }
        }
        if self.peek() == Some('"') {
            n.text = Some(self.quoted()?);
            while self
                .peek()
                .is_some_and(|c| c == ' ' || c == '\t' || c == '\r')
            {
                self.bump();
            }
        }
        if self.peek() == Some('{') {
            self.bump();
            n.children = self.nodes(true, depth + 1)?;
        }
        if is_void(&n.tag) && (n.text.is_some() || !n.children.is_empty()) {
            return Err(self.error("void elements cannot have content"));
        }
        Ok(n)
    }
}
fn is_void(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn render(nodes: &[Node], depth: usize, out: &mut String) {
    for n in nodes {
        if n.tag.is_empty() {
            out.push_str(&escape(n.text.as_deref().unwrap_or("")));
            continue;
        }
        let pad = "  ".repeat(depth);
        out.push_str(&format!("{pad}<{}", n.tag));
        if let Some(id) = &n.id {
            out.push_str(&format!(" id=\"{}\"", escape(id)));
        }
        if !n.classes.is_empty() {
            out.push_str(&format!(" class=\"{}\"", escape(&n.classes.join(" "))));
        }
        for (k, v) in &n.attrs {
            out.push_str(&format!(" {k}=\"{}\"", escape(v)));
        }
        out.push('>');
        if let Some(t) = &n.text {
            out.push_str(&escape(t));
        }
        if !n.children.is_empty() {
            if n.children.iter().any(|c| c.tag.is_empty()) {
                for child in &n.children {
                    if child.tag.is_empty() {
                        out.push_str(&escape(child.text.as_deref().unwrap_or("")));
                    } else {
                        let mut part = String::new();
                        render(std::slice::from_ref(child), 0, &mut part);
                        out.push_str(part.trim_end_matches('\n'));
                    }
                }
            } else {
                out.push('\n');
                render(&n.children, depth + 1, out);
                out.push_str(&pad);
            }
        }
        if !is_void(&n.tag) {
            out.push_str(&format!("</{}>", n.tag));
        }
        out.push('\n');
    }
}
fn compile(input: &str, fragment: bool) -> Result<String, String> {
    let nodes = Parser::new(input).nodes(false, 0)?;
    let mut out = String::new();
    if !fragment {
        out.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <script src=\"https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4\"></script>\n  <title>Document</title>\n</head>\n<body>\n");
    }
    render(&nodes, if fragment { 0 } else { 1 }, &mut out);
    if !fragment {
        out.push_str("</body>\n</html>\n");
    }
    Ok(out)
}
fn run() -> Result<(), String> {
    let mut input = None;
    let mut output = None;
    let mut fragment = false;
    let mut tokens = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!(
                    "Usage: hm [INPUT] [-o OUTPUT] [--fragment] [--tokens]\nReads stdin when INPUT is omitted; writes HTML to stdout by default. --tokens reports token estimates."
                );
                return Ok(());
            }
            "--fragment" => fragment = true,
            "--tokens" => tokens = true,
            "-o" | "--output" => output = Some(args.next().ok_or("missing output path")?),
            _ if arg.starts_with('-') && arg != "-" => {
                return Err(format!("unknown option: {arg}"));
            }
            _ if input.is_none() => input = Some(arg),
            _ => return Err("only one input file is supported".into()),
        }
    }
    let source = match input.as_deref() {
        None | Some("-") => {
            let mut s = String::new();
            io::stdin()
                .read_to_string(&mut s)
                .map_err(|e| e.to_string())?;
            s
        }
        Some(path) => fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?,
    };
    let started = Instant::now();
    let html = compile(&source, fragment)?;
    let elapsed = started.elapsed();
    match output.as_deref() {
        None | Some("-") => io::stdout()
            .write_all(html.as_bytes())
            .map_err(|e| e.to_string())?,
        Some(path) => fs::write(path, &html).map_err(|e| format!("{path}: {e}"))?,
    }
    eprintln!(
        "Characters: {} before, {} after",
        source.chars().count(),
        html.chars().count()
    );
    eprintln!(
        "Parse and render time: {:.3} ms",
        elapsed.as_secs_f64() * 1000.0
    );
    if let Some(path) = output.as_deref().filter(|p| *p != "-") {
        let location = fs::canonicalize(path).map_err(|e| format!("{path}: {e}"))?;
        eprintln!("Saved to: {}", location.display());
    }
    if !tokens {
        return Ok(());
    }
    let tokenizer = tiktoken_rs::o200k_base().map_err(|e| format!("tokenizer: {e}"))?;
    let count = |s: &str| tokenizer.encode_ordinary(s).len();
    let source_tokens = count(&source);
    let html_tokens = count(&html);
    let skill_tokens = count(include_str!("../skills/hol-markup/SKILL.md"));
    let report = |total: usize| {
        if html_tokens == 0 {
            return "n/a (empty HTML)".to_string();
        }
        let difference = html_tokens as i64 - total as i64;
        format!(
            "{difference:+} tokens ({:+.1}%)",
            difference as f64 * 100.0 / html_tokens as f64
        )
    };
    eprintln!("Token estimate (o200k_base; model tokenizers vary):");
    eprintln!("  HTML: {html_tokens}; .hm: {source_tokens}; skill: {skill_tokens}");
    eprintln!("  Savings without skill: {}", report(source_tokens));
    eprintln!(
        "  Savings including skill once: {}",
        report(source_tokens + skill_tokens)
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("hm: {e}");
        process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_and_escaped() {
        let s = compile(
            "div.bg-red-300.rounded-md {\np.text-md \"hello & <world>\"\n}",
            true,
        )
        .unwrap();
        assert_eq!(
            s,
            "<div class=\"bg-red-300 rounded-md\">\n  <p class=\"text-md\">hello &amp; &lt;world&gt;</p>\n</div>\n"
        );
    }
    #[test]
    fn attrs_and_void() {
        assert_eq!(
            compile("img.w-full[src=\"a&b\"][alt=\"hi\"]", true).unwrap(),
            "<img class=\"w-full\" src=\"a&amp;b\" alt=\"hi\">\n"
        );
    }
    #[test]
    fn arbitrary_values_and_attributes() {
        let source = r##"div.w-[2.5rem].bg-[#123456].hover:bg-[rgb(0,0,0)].[&>p]:text-[length:var(--size)][title="hello"]"##;
        assert_eq!(
            compile(source, true).unwrap(),
            "<div class=\"w-[2.5rem] bg-[#123456] hover:bg-[rgb(0,0,0)] [&amp;&gt;p]:text-[length:var(--size)]\" title=\"hello\"></div>\n"
        );
        assert!(
            compile("div.w-[12px", true)
                .unwrap_err()
                .contains("unterminated arbitrary value")
        );
    }
    #[test]
    fn href_shorthand() {
        assert_eq!(
            compile(r#"a.btn$"/about" "About""#, true).unwrap(),
            "<a class=\"btn\" href=\"/about\">About</a>\n"
        );
        assert_eq!(
            compile(r#"a$"https://example.com/?a=1&b=2" "Link""#, true).unwrap(),
            "<a href=\"https://example.com/?a=1&amp;b=2\">Link</a>\n"
        );
        assert!(compile("a$", true).is_err());
        assert!(compile("a$/about", true).is_err());
        assert!(compile("a href=/about", true).is_err());
        assert_eq!(
            compile(r##"a$"/x?#a&b=%20!$[x]{y}""##, true).unwrap(),
            "<a href=\"/x?#a&amp;b=%20!$[x]{y}\"></a>\n"
        );
        assert!(compile(r#"a$/x[href="/y"]"#, true).is_err());
    }
    #[test]
    fn image_shorthand_and_id() {
        assert_eq!(
            compile(r#"img%"photo.jpg"&"Portrait of Alex"[id="photo"]"#, true).unwrap(),
            "<img src=\"photo.jpg\" alt=\"Portrait of Alex\" id=\"photo\">\n"
        );
        assert!(compile(r#"img%"photo"[src="other"]"#, true).is_err());
        assert!(compile(r#"img%"photo"&"#, true).is_err());
        assert_eq!(
            compile("div#hello", true).unwrap(),
            "<div id=\"hello\"></div>\n"
        );
        assert!(compile(r#"div#hello[id="other"]"#, true).is_err());
    }
    #[test]
    fn errors() {
        assert!(
            compile("div { p \"x\"", true)
                .unwrap_err()
                .contains("expected '}'")
        );
        assert!(compile("br \"x\"", true).is_err());
    }
    #[test]
    fn compact_div_and_empty_attribute() {
        assert_eq!(
            compile(".panel#intro { input[type=\"email\"][required] }", true).unwrap(),
            "<div id=\"intro\" class=\"panel\">\n  <input type=\"email\" required=\"\">\n</div>\n"
        );
        assert!(compile(".x[class]", true).is_err());
        assert!(compile("#a[id]", true).is_err());
        assert!(compile("input[required][required]", true).is_err());
        assert_eq!(compile("input!", true).unwrap(), "<input required=\"\">\n");
        assert!(compile("input![required]", true).is_err());
        assert!(compile("input[required]!", true).is_err());
    }
    #[test]
    fn spaced_attributes_and_siblings() {
        assert_eq!(
            compile(
                "input.field type=\"email\" name=\"a&b\"\ndiv p \"hi\"",
                true
            )
            .unwrap(),
            "<input class=\"field\" type=\"email\" name=\"a&amp;b\">\n<div></div>\n<p>hi</p>\n"
        );
        assert!(compile("input[type=\"x\"] type=\"y\"", true).is_err());
        assert!(compile("div#x id=\"y\"", true).is_err());
        assert_eq!(
            compile("input type=email name=address", true).unwrap(),
            "<input type=\"email\" name=\"address\">\n"
        );
        assert_eq!(
            compile("div title=hello { p \"hi\" }", true).unwrap(),
            "<div title=\"hello\">\n  <p>hi</p>\n</div>\n"
        );
        assert!(compile("input type=", true).is_err());
        assert!(compile("div class=\"x\"", true).is_err());
    }
}
