use rocket::form::Form;
use rocket::http::{Cookie, SameSite,CookieJar};
use rocket::response::Redirect;
use rocket::State;
use rocket::time::{Duration, OffsetDateTime};
use rocket_dyn_templates::{context,Template};
use crate::config::Config;
use crate::{data_base_manager, error};
use crate::upload::{check_string_for_bad_actors};

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::from("user"));
    Redirect::to(uri!("/submit/login"))
}
#[get("/login")]
pub async fn login_form(_config:&State<Config>) -> Template {
    Template::render("LoginForm", context! {})
}

#[derive(FromForm)]
pub struct LoginForm<'r> {
    pub username: &'r str,
    pub password: &'r str,
}

#[post("/login/send", data = "<form>")]
pub async fn login_data(_n: &State<Config>, form: Form<LoginForm<'_>>, cookies:  &CookieJar<'_>) -> Redirect {
    if let Some(_) = cookies.get("user") {
        return Redirect::to(uri!("/submit/upload"));
    }
    
    let LoginForm {
        username,
        password
    } = form.into_inner();


    let is_username_ok: bool = check_string_for_bad_actors::<false>(&username);
    let is_password_ok: bool = check_string_for_bad_actors::<false>(&password);
    if !is_username_ok {
        return error::page("username has bad charaters in it so is malformed.");
    }

    if !is_password_ok {
        return error::page("password has bad charaters in it so is malformed. if you have added anything not A-Za-z-0-9 this is currently not supported");
    }
    
    let is_login_ok = data_base_manager::login_in(username, password).await;
    
    if is_login_ok.is_empty() {
        cookies.add(
            Cookie::build(("user", username.to_string()))
            .expires(OffsetDateTime::now_utc() + Duration::days(1))
            .same_site(SameSite::Lax));
        return Redirect::to(uri!("/submit/upload"));
    }
    
    Redirect::to(uri!("/submit/login"))
}
