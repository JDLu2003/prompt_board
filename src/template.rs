use regex::Regex;
use std::collections::HashSet;

pub fn extract_variables(content: &str) -> Vec<String> {
    let re = Regex::new(r"\[([^\[\]\n]+)\]").expect("valid variable regex");
    let mut seen = HashSet::new();
    let mut vars = Vec::new();

    for cap in re.captures_iter(content) {
        let name = cap[1].trim();
        if !name.is_empty() && seen.insert(name.to_owned()) {
            vars.push(name.to_owned());
        }
    }

    vars
}

pub fn render_template(content: &str, values: &[(String, String)]) -> String {
    let mut rendered = content.to_owned();
    for (name, value) in values {
        if value.is_empty() {
            continue;
        }
        rendered = rendered.replace(&format!("[{}]", name), value);
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_unique_variables_in_order() {
        let vars = extract_variables("Hi [name], summarize [topic] for [name].");
        assert_eq!(vars, vec!["name", "topic"]);
    }

    #[test]
    fn renders_template_values() {
        let rendered = render_template(
            "Write to [recipient] about [topic].",
            &[
                ("recipient".to_owned(), "Alice".to_owned()),
                ("topic".to_owned(), "launch".to_owned()),
            ],
        );
        assert_eq!(rendered, "Write to Alice about launch.");
    }

    #[test]
    fn keeps_placeholder_for_empty_values() {
        let rendered = render_template(
            "请总结[主题]，风格为[风格]。",
            &[
                ("主题".to_owned(), "Rust".to_owned()),
                ("风格".to_owned(), String::new()),
            ],
        );
        assert_eq!(rendered, "请总结Rust，风格为[风格]。");
    }
}
