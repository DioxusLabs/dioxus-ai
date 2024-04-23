use scraper::{ElementRef, Node};

const IMPORTANT_ATTRIBUTES: &[&str] = &["id", "href", "alt", "title", "aria-*", "role", "type"];
const IMPORTANT_ELEMENTS: &[&str] = &[
    "a", "img", "p", "h1", "h2", "h3", "ul", "ol", "li", "table", "tr", "td", "button", "input",
    "textarea", "select", "option", "form", "label",
];

pub fn get_clean_html(node: &kalosm::language::Node) -> anyhow::Result<String> {
    let visible_html = node.outer_html_visible()?;

    Ok(clean_html(&visible_html))
}

fn clean_html(fragment: &str) -> String {
    let mut result = String::new();

    let html = scraper::html::Html::parse_fragment(fragment);

    visit_element(html.root_element(), &mut result);

    result
}

fn visit_element(element: ElementRef, result: &mut String) {
    let value = element.value();
    let lowercase_name = value.name().to_lowercase();

    let is_important = IMPORTANT_ELEMENTS.contains(&lowercase_name.as_str());
    if is_important {
        result.push('<');
        result.push_str(&lowercase_name);

        for (attribute, value) in value.attrs() {
            if IMPORTANT_ATTRIBUTES.contains(&attribute) {
                result.push(' ');
                result.push_str(attribute);
                result.push('=');
                result.push('"');
                result.push_str(value);
                result.push('"');
            }
        }

        result.push('>');
    }

    for child in element.children() {
        match child.value() {
            Node::Element(_) => visit_element(ElementRef::wrap(child).unwrap(), result),
            Node::Text(t) => result.push_str(t),
            _ => {}
        }
    }

    if is_important {
        result.push('<');
        result.push('/');
        result.push_str(&lowercase_name);
        result.push('>');
    }
}
