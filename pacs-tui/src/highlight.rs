use ratatui::text::{Line, Span};

use crate::theme::Theme;

pub fn highlight_shell<'a>(input: &'a str, theme: &Theme) -> Vec<Line<'a>> {
    input
        .lines()
        .map(|line| highlight_line(line, theme))
        .collect()
}

#[allow(clippy::too_many_lines)]
fn highlight_line<'a>(line: &'a str, theme: &Theme) -> Line<'a> {
    let mut spans: Vec<Span<'a>> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut expect_command = true; // true at start and after pipe/semicolon

    while i < len {
        let ch = chars[i];

        // Comments
        if ch == '#' {
            let start = i;
            let rest: String = chars[start..].iter().collect();
            spans.push(Span::styled(rest, theme.sh_comment));
            break;
        }

        // Strings (double quotes)
        if ch == '"' {
            let start = i;
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < len {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < len {
                i += 1; // include closing quote
            }
            let s: String = chars[start..i].iter().collect();
            spans.push(Span::styled(s, theme.sh_string));
            continue;
        }

        // Strings (single quotes)
        if ch == '\'' {
            let start = i;
            i += 1;
            while i < len && chars[i] != '\'' {
                i += 1;
            }
            if i < len {
                i += 1; // include closing quote
            }
            let s: String = chars[start..i].iter().collect();
            spans.push(Span::styled(s, theme.sh_string));
            continue;
        }

        // Variables ($VAR or ${VAR})
        if ch == '$' {
            let start = i;
            i += 1;
            if i < len && chars[i] == '{' {
                i += 1;
                while i < len && chars[i] != '}' {
                    i += 1;
                }
                if i < len {
                    i += 1; // include closing brace
                }
            } else {
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
            }
            let s: String = chars[start..i].iter().collect();
            spans.push(Span::styled(s, theme.sh_variable));
            continue;
        }

        // Operators (|, >, >>, <, <<, &&, ||, ;)
        if ch == '|' || ch == '>' || ch == '<' || ch == '&' || ch == ';' {
            let start = i;
            i += 1;
            // Handle double operators
            if i < len
                && (chars[i] == ch
                    || (ch == '>' && chars[i] == '>')
                    || (ch == '<' && chars[i] == '<'))
            {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            spans.push(Span::styled(s, theme.sh_operator));
            // After pipe or semicolon, expect a new command
            if ch == '|' || ch == ';' {
                expect_command = true;
            }
            continue;
        }

        // Flags (--flag or -f)
        if ch == '-' && (i == 0 || chars[i - 1].is_whitespace()) {
            let start = i;
            i += 1;
            if i < len && chars[i] == '-' {
                i += 1;
            }
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
                i += 1;
            }
            if i > start + 1 {
                let s: String = chars[start..i].iter().collect();
                spans.push(Span::styled(s, theme.sh_flag));
                continue;
            }
            i = start; // reset, not a flag
        }

        // Skip whitespace
        if ch.is_whitespace() {
            let start = i;
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            spans.push(Span::raw(s));
            continue;
        }

        // Regular text (collect until special character or whitespace)
        let start = i;
        while i < len {
            let c = chars[i];
            if c.is_whitespace()
                || c == '#'
                || c == '"'
                || c == '\''
                || c == '$'
                || c == '|'
                || c == '>'
                || c == '<'
                || c == '&'
                || c == ';'
            {
                break;
            }
            if c == '-' && (i == 0 || chars[i - 1].is_whitespace()) {
                break;
            }
            i += 1;
        }
        if i > start {
            let s: String = chars[start..i].iter().collect();
            if expect_command {
                spans.push(Span::styled(s, theme.sh_command));
                expect_command = false;
            } else {
                spans.push(Span::raw(s));
            }
        }
    }

    Line::from(spans)
}
