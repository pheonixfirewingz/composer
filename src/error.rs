use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};
use urlencoding::encode;

#[get("/?<error>")]
pub fn _error(error: Option<&str>) -> Template {
    let mut error_msg: String = String::from("");
    if let Some(error) = error {
        error_msg = error.to_string();
    }
    Template::render("SubmitResponse", context! {
        failed: true,
        message: error_msg
    })
}

#[inline(always)]
pub fn page(msg: &str) -> Redirect {
    let encoded_msg = encode(msg);
    Redirect::to(format!("/submit/error?error={}", encoded_msg))
}