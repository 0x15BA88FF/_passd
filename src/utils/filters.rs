use regex::Regex;

#[derive(PartialEq)]
pub enum Method {
    MatchCase,
    MatchWord,
    MatchRegex,
}

pub fn match_filter(methods: Vec<Method>) -> impl Fn(&str) -> bool {
    move |input: &str| {
        let mut result = true;

        for method in &methods {
            match method {
                Method::MatchCase => {
                    result = result && !input.is_empty();
                }
                Method::MatchWord => {
                    let pattern = format!(r"\b{}\b", regex::escape(input));
                    if let Ok(regex) = Regex::new(&pattern) {
                        result = result && regex.is_match(input);
                    }
                }
                Method::MatchRegex => {
                    result = result && Regex::new(input).is_ok();
                }
            }
        }

        result
    }
}
