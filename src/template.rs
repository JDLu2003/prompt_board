use regex::Regex;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateVariable {
    pub name: String,
    pub default_value: Option<String>,
    pub placeholder: String,
}

#[allow(dead_code)]
pub fn extract_variables(content: &str) -> Vec<String> {
    extract_template_variables(content)
        .into_iter()
        .map(|variable| variable.name)
        .collect()
}

pub fn extract_template_variables(content: &str) -> Vec<TemplateVariable> {
    let re = Regex::new(r"\[([^\[\]\n]+)\]").expect("valid variable regex");
    let mut seen = HashSet::new();
    let mut vars = Vec::new();

    for cap in re.captures_iter(content) {
        let raw = cap[1].trim();
        let (name, default_value) = parse_variable(raw);
        if !name.is_empty() && seen.insert(name.to_owned()) {
            vars.push(TemplateVariable {
                name: name.to_owned(),
                default_value,
                placeholder: cap[0].to_owned(),
            });
        }
    }

    vars
}

#[allow(dead_code)]
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

pub fn render_template_with_placeholders(
    content: &str,
    variables: &[TemplateVariable],
    values: &[(String, String)],
) -> String {
    let mut rendered = content.to_owned();
    for variable in variables {
        let Some((_, value)) = values.iter().find(|(name, _)| name == &variable.name) else {
            continue;
        };
        if value.is_empty() {
            continue;
        }
        rendered = rendered.replace(&variable.placeholder, value);
    }
    rendered
}

fn parse_variable(raw: &str) -> (&str, Option<String>) {
    let Some((name, default_value)) = raw.split_once('|') else {
        return (raw.trim(), None);
    };
    let default_value = default_value.trim();
    (
        name.trim(),
        (!default_value.is_empty()).then(|| default_value.to_owned()),
    )
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
    fn extracts_default_values() {
        let vars = extract_template_variables("请写给[收件人|张三]，语气为[语气|正式]。");
        assert_eq!(
            vars,
            vec![
                TemplateVariable {
                    name: "收件人".to_owned(),
                    default_value: Some("张三".to_owned()),
                    placeholder: "[收件人|张三]".to_owned(),
                },
                TemplateVariable {
                    name: "语气".to_owned(),
                    default_value: Some("正式".to_owned()),
                    placeholder: "[语气|正式]".to_owned(),
                },
            ]
        );
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
        let vars = extract_template_variables("请总结[主题]，风格为[风格|专业]。");
        let rendered = render_template_with_placeholders(
            "请总结[主题]，风格为[风格|专业]。",
            &vars,
            &[
                ("主题".to_owned(), "Rust".to_owned()),
                ("风格".to_owned(), String::new()),
            ],
        );
        assert_eq!(rendered, "请总结Rust，风格为[风格|专业]。");
    }
}
