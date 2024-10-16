use reqwest::Response;
use xmlparser::{Tokenizer, Token};

#[inline(always)]
fn check_for_error<'a>(xml: &str) -> String {
    let mut tokenizer = Tokenizer::from(xml);
    let mut in_error: bool = false;
    let mut error_message: String = String::new();

    while let Some(token) = tokenizer.next() {
        match token {
            Ok(Token::ElementStart { local, .. }) => {
                if local.as_str() == "error" {
                    in_error = true;
                }
            }
            Ok(Token::Attribute { local, value, .. }) => {
                if in_error {
                    match local.as_str() {
                        "message" => error_message = String::from(value.as_str()),
                        _ => {}
                    }
                }
            }
            Ok(Token::ElementEnd { .. }) => {
                if in_error {
                    in_error = false;
                }
            }
            Err(_) => {
                return error_message;
            }
            _ => {}
        }
    }
    error_message
}

#[inline(always)]
const fn server() -> &'static str {
    if cfg!(debug_assertions) {
        "192.168.0.2"
    } else {
        "127.0.0.1"
    }
}

pub async fn login_in<'a>(username: & str, password: & str) -> String {
    let request = reqwest::get(format!("http://{}:4533/rest/ping.view?u={}&p={}&v=1.16.1&c=composer",server(),username,password)).await;
    let response: Response = request.unwrap();
    if response.status() == reqwest::StatusCode::OK || response.status() == reqwest::StatusCode::NO_CONTENT {
        let body = response.text().await;
        let data: String = body.unwrap();
        return check_for_error(&data);
    }
    String::from("500 bad api call request")
}
