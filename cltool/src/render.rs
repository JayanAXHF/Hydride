pub fn with_heading(changelog: &str, heading: Option<&str>) -> String {
    match heading {
        Some(heading) if !heading.trim().is_empty() => format!("{heading}\n\n{changelog}"),
        _ => changelog.to_owned(),
    }
}

pub fn chunk_for_discord(input: &str, max_chars: usize) -> Vec<String> {
    let input = input.trim();
    if input.is_empty() {
        return vec![String::from("_No changelog entries found._")];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for line in input.lines() {
        if line.len() <= max_chars {
            if current.is_empty() {
                current.push_str(line);
            } else if current.len() + 1 + line.len() <= max_chars {
                current.push('\n');
                current.push_str(line);
            } else {
                chunks.push(std::mem::take(&mut current));
                current.push_str(line);
            }
            continue;
        }

        if !current.is_empty() {
            chunks.push(std::mem::take(&mut current));
        }

        let mut start = 0;
        while start < line.len() {
            let mut end = (start + max_chars).min(line.len());
            while end > start && !line.is_char_boundary(end) {
                end -= 1;
            }
            if end == start {
                end = (start + 1).min(line.len());
                while end < line.len() && !line.is_char_boundary(end) {
                    end += 1;
                }
            }
            chunks.push(line[start..end].to_owned());
            start = end;
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    if chunks.is_empty() {
        vec![String::from("_No changelog entries found._")]
    } else {
        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_long_text() {
        let text = "a\nb\nc\nd";
        let chunks = chunk_for_discord(text, 3);
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn uses_placeholder_for_empty_text() {
        let chunks = chunk_for_discord("", 10);
        assert_eq!(chunks, vec![String::from("_No changelog entries found._")]);
    }
}
