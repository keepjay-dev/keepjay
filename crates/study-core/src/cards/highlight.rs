// SPDX-License-Identifier: AGPL-3.0-or-later
//! Syntax highlighting for code blocks on cards (CODE-001, SRS-008).
//!
//! Code is split into tokens of a few broad [`Kind`]s, not colours, so each surface picks
//! its own: the TUI uses the terminal's palette, a web view would use CSS classes (08 §4).
//! Grammars are syntect's built-in set, about 75 languages, parsed with a pure-Rust regex
//! engine. They load on first use, so commands without code blocks never pay for them.

use std::sync::OnceLock;

use syntect::parsing::{ParseState, Scope, ScopeStack, ScopeStackOp, SyntaxSet};

/// What a token is, broadly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Plain,
    Comment,
    Keyword,
    String,
    /// Numbers, `true`, `None`, placeholders such as `{n}`.
    Constant,
    Type,
    Function,
}

/// A run of code of one kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    pub kind: Kind,
}

fn syntaxes() -> &'static SyntaxSet {
    static SET: OnceLock<SyntaxSet> = OnceLock::new();
    SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// Scope prefixes and their kinds. The innermost scope that matches wins, and within one
/// scope the first matching rule, so `keyword.operator` stays plain.
fn rules() -> &'static [(Scope, Kind)] {
    static RULES: OnceLock<Vec<(Scope, Kind)>> = OnceLock::new();
    RULES.get_or_init(|| {
        [
            ("comment", Kind::Comment),
            ("keyword.operator", Kind::Plain),
            ("punctuation", Kind::Plain),
            ("string", Kind::String),
            ("constant.character", Kind::String),
            ("constant", Kind::Constant),
            ("keyword", Kind::Keyword),
            ("storage", Kind::Keyword),
            ("entity.name.function", Kind::Function),
            ("variable.function", Kind::Function),
            ("support.function", Kind::Function),
            ("support.macro", Kind::Function),
            ("entity.name", Kind::Type),
            ("entity.other.inherited-class", Kind::Type),
            ("support.type", Kind::Type),
            ("support.class", Kind::Type),
        ]
        .into_iter()
        .map(|(s, k)| (Scope::new(s).expect("valid scope"), k))
        .collect()
    })
}

fn kind_of(stack: &ScopeStack) -> Kind {
    for scope in stack.as_slice().iter().rev() {
        if let Some((_, kind)) = rules().iter().find(|(p, _)| p.is_prefix_of(*scope)) {
            // Punctuation inside a string or comment takes the string's or comment's kind.
            if *kind == Kind::Plain && stack.as_slice().iter().any(|s| enclosing(s).is_some()) {
                return stack
                    .as_slice()
                    .iter()
                    .rev()
                    .find_map(enclosing)
                    .unwrap_or(Kind::Plain);
            }
            return *kind;
        }
    }
    Kind::Plain
}

fn enclosing(scope: &Scope) -> Option<Kind> {
    let [comment, string] =
        [Scope::new("comment"), Scope::new("string")].map(|s| s.expect("valid"));
    if comment.is_prefix_of(*scope) {
        Some(Kind::Comment)
    } else if string.is_prefix_of(*scope) {
        Some(Kind::String)
    } else {
        None
    }
}

/// The grammar's name for a fence's language word, e.g. `rust`, `py`, `sh`.
fn token_for(lang: &str) -> String {
    let lang = lang.to_ascii_lowercase();
    match lang.as_str() {
        // Close enough for short snippets; the built-in set has no grammars for these.
        "ts" | "typescript" | "tsx" | "jsx" | "mjs" | "cjs" | "javascript" => "js".into(),
        "shell" | "console" | "zsh" | "fish" => "sh".into(),
        "golang" => "go".into(),
        "csharp" | "c#" => "cs".into(),
        "c++" => "cpp".into(),
        "yml" => "yaml".into(),
        _ => lang,
    }
}

/// Tokens for each line of a code block, or `None` for an unknown language (shown plain).
pub fn highlight(lang: &str, lines: &[String]) -> Option<Vec<Vec<Token>>> {
    let set = syntaxes();
    let syntax = set.find_syntax_by_token(&token_for(lang))?;
    if syntax.name == "Plain Text" {
        return None;
    }
    let mut state = ParseState::new(syntax);
    let mut stack = ScopeStack::new();
    let mut out = Vec::with_capacity(lines.len());
    for line in lines {
        let text = format!("{line}\n");
        let ops = state.parse_line(&text, set).ok()?;
        let mut tokens: Vec<Token> = Vec::new();
        let mut from = 0;
        let end = [(line.len(), ScopeStackOp::Noop)];
        for (at, op) in ops.iter().chain(end.iter()) {
            let at = (*at).min(line.len());
            if at > from {
                push(&mut tokens, &line[from..at], kind_of(&stack));
                from = at;
            }
            stack.apply(op).ok()?;
        }
        out.push(tokens);
    }
    Some(out)
}

fn push(tokens: &mut Vec<Token>, text: &str, kind: Kind) {
    // Whitespace joins its neighbour, so tokens stay few.
    let kind = if text.trim().is_empty() {
        tokens.last().map_or(Kind::Plain, |t| t.kind)
    } else {
        kind
    };
    match tokens.last_mut() {
        Some(last) if last.kind == kind => last.text.push_str(text),
        _ => tokens.push(Token {
            text: text.to_string(),
            kind,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(lang: &str, code: &str) -> Vec<(String, Kind)> {
        let lines: Vec<String> = code.lines().map(str::to_string).collect();
        highlight(lang, &lines)
            .expect("known language")
            .into_iter()
            .flatten()
            .filter(|t| !t.text.trim().is_empty())
            .map(|t| (t.text.trim().to_string(), t.kind))
            .collect()
    }

    fn kind_of_word(tokens: &[(String, Kind)], word: &str) -> Kind {
        tokens
            .iter()
            .find(|(t, _)| t == word || t.split_whitespace().any(|w| w == word))
            .unwrap_or_else(|| panic!("{word} not in {tokens:?}"))
            .1
    }

    #[test]
    fn rust_tokens() {
        let t = kinds(
            "rust",
            "pub fn answer() -> u32 {\n    let n = 42; // why\n    println!(\"{n}\");\n    n\n}",
        );
        assert_eq!(kind_of_word(&t, "fn"), Kind::Keyword);
        assert_eq!(kind_of_word(&t, "answer"), Kind::Function);
        assert_eq!(kind_of_word(&t, "42"), Kind::Constant);
        assert_eq!(kind_of_word(&t, "// why"), Kind::Comment);
        assert_eq!(kind_of_word(&t, "println!"), Kind::Function);
        assert_eq!(kind_of_word(&t, "="), Kind::Plain);
    }

    #[test]
    fn strings_keep_their_quotes() {
        let t = kinds("py", "x = 'a b'");
        assert!(t.contains(&("'a b'".into(), Kind::String)), "{t:?}");
    }

    #[test]
    fn python_types_and_aliases() {
        let t = kinds("python", "class A(B):\n    def f(self) -> int: return None");
        assert_eq!(kind_of_word(&t, "class"), Kind::Keyword);
        assert_eq!(kind_of_word(&t, "A"), Kind::Type);
        assert_eq!(kind_of_word(&t, "int"), Kind::Type);
        assert_eq!(kind_of_word(&t, "None"), Kind::Constant);
        // Fence words other than the grammar's own name.
        assert!(highlight("ts", &["const x = 1".into()]).is_some());
        assert!(highlight("Shell", &["echo hi".into()]).is_some());
    }

    #[test]
    fn whitespace_is_kept_exactly() {
        let lines = vec!["\tif x:".to_string(), "".into(), "        pass  ".into()];
        let out = highlight("py", &lines).unwrap();
        let back: Vec<String> = out
            .iter()
            .map(|l| l.iter().map(|t| t.text.as_str()).collect())
            .collect();
        assert_eq!(back, lines);
    }

    #[test]
    fn unknown_languages_are_plain() {
        assert!(highlight("klingon", &["x".into()]).is_none());
        assert!(highlight("text", &["x".into()]).is_none());
    }

    #[test]
    fn state_carries_across_lines() {
        let t = kinds("rust", "/* a\n   b */ fn x() {}");
        assert_eq!(kind_of_word(&t, "b */"), Kind::Comment);
        assert_eq!(kind_of_word(&t, "fn"), Kind::Keyword);
    }
}
