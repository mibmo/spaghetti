#![feature(proc_macro_hygiene, decl_macro)]

extern crate openssl;
#[macro_use]
extern crate rocket;

use rocket::{
    http::Status,
    request::{Form, FromForm},
    response::Redirect,
};
use rocket_contrib::json::Json;

use anyhow::{anyhow, bail};
use serde::Serialize;
use url::Url;

use spaghetti::RedirectDb as DbConn;

const ALLOWED_URL_SCHEMES: [&str; 3] = ["http", "https", "ftp"];
const BASE64_ENCODE_CONFIG: base64::Config = base64::URL_SAFE_NO_PAD;

#[derive(FromForm)]
struct NewRedirectForm {
    url: String,
}

#[derive(Serialize)]
struct RedirectResponse {
    error: String,
    id: String,
}

#[get("/")]
fn index() -> &'static str {
    "Hello there!\nGeneral Kenobi..."
}

// @TODO: remove endpoint
#[get("/all")]
fn show_all_redirects(conn: DbConn) -> String {
    match conn.get_all_redirects() {
        Err(e) => format!("Database error: {}", e),
        Ok(redirects) => redirects
            .into_iter()
            .map(|req| (req.id, req.url))
            .map(|(id, url)| format!("ID: {:5}\t URL: {}", id, url))
            .collect::<Vec<String>>()
            .join("\n"),
    }
}

#[post("/new", data = "<new_redirect>")]
fn new_redirect(conn: DbConn, new_redirect: Form<NewRedirectForm>) -> Json<RedirectResponse> {
    // this isn't a very clean way to return
    // fallible json but it'll suffice

    let error_response = RedirectResponse {
        error: "failed to create redirect".to_string(),
        id: "".to_string(),
    };

    let url = match parse_url(&new_redirect.url) {
        Err(_) => return Json(error_response),
        Ok(url) => url,
    };

    match conn.create_redirect(&url.to_string()) {
        Err(_) => Json(error_response),
        Ok(id) => Json(RedirectResponse {
            error: "".to_string(),
            id: id.to_string(),
        })
    }
}

/// ID is a base64-encoded int32 value
#[get("/<id>")]
fn redirector(conn: DbConn, id: String) -> Redirect {
    match conn.get_redirect_with_id(&id) {
        Err(e) => Redirect::to(uri!(index)), // redirect to index when not found
        Ok(redirect) => Redirect::to(redirect.url),
    }
}

/// Parses a URL and performs validation checks, such that it can be reasonably trusted.
// This definitely isn't tamperproof, but I deem it _good enough._
fn parse_url(url: &str) -> anyhow::Result<Url> {
    let url = match Url::parse(url) {
        Ok(url) => url,
        Err(_e) => Url::parse(&format!("https://{}", url))?,
    };

    // is scheme valid
    if !ALLOWED_URL_SCHEMES
        .iter()
        .any(|&scheme| scheme == url.scheme())
    {
        bail!("URL scheme {} is not a valid scheme", url.scheme());
    }

    let no_domain_error = anyhow!("URL does not have a domain");

    // slight magic ensues
    match url.host_str() {
        None => bail!("No host specified"),
        Some(host) => match host.find('.') {
            Some(v) if v == 0 || v == host.len() - 1 => bail!("No TLD specified"), // if the period is the first or last character
            None => return Err(no_domain_error),
            _ => (),
        },
    }

    if url.domain().is_none() {
        return Err(no_domain_error);
    }

    Ok(url)
}

fn main() {
    let routes = routes![index, show_all_redirects, new_redirect, redirector,];

    rocket::ignite()
        .mount("/", routes)
        .attach(DbConn::fairing())
        .launch();
}
