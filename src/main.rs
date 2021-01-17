#![feature(proc_macro_hygiene, decl_macro)]

extern crate openssl;
#[macro_use] extern crate rocket;

use rocket::{request::Form, http::Status};
use rocket_contrib::json::Json;

use url::Url;
use anyhow::{bail, anyhow};


use spaghetti::{
    RedirectDb as DbConn,
    models::NewRedirect,
};

const ALLOWED_URL_SCHEMES: [&str; 3] = ["http", "https", "ftp"];
const BASE64_ENCODE_CONFIG: base64::Config = base64::URL_SAFE_NO_PAD;

#[get("/")]
fn index() -> &'static str {
    "Hello there!\nGeneral Kenobi..."
}

#[get("/all")]
// @TODO: use webnum
fn show_all_redirects(conn: DbConn) -> String {
    match conn.get_all_redirects() {
        Err(e) => format!("Database error: {}", e),
        Ok(redirects) => redirects.into_iter()
            .map(|req| (req.id, req.url))
            .map(|(id, url)| (base64::encode_config(id.to_string(), BASE64_ENCODE_CONFIG), url))
            .map(|(id, url)| format!("ID: {:4}\t URL: {}", id, url))
            .collect::<Vec<String>>()
            .join("\n"),
    }

}

#[post("/new", data = "<new_redirect>")]
// @TODO: use webnum
fn new_redirect(conn: DbConn, new_redirect: Form<NewRedirect>) -> Result<String, Status> {
    let url = match parse_url(&new_redirect.url) {
        Ok(url) => url,
        Err(e) => return Ok(format!("Error: {}", e))//return Err(Status::BadRequest), // @TODO: Return error page, maybe?
    };

    match conn.create_redirect(&url.to_string()) {
        Err(_) => return Err(Status::InternalServerError),
        Ok(id) => {
            let encoded = base64::encode_config(id.to_string(), BASE64_ENCODE_CONFIG);
            Ok(encoded)
        },
    }
}

/// ID is a base64-encoded int32 value
#[get("/<encoded_id>")]
fn redirector(conn: DbConn, encoded_id: String) -> String {
    let id = match decode_id(encoded_id) {
        Ok(n) => n,
        Err(_) => return format!("Not a valid ID"),
    };

    match conn.get_redirect_with_id(id) {
        Err(e) => return format!("Database error: {}", e),
        Ok(redirect) => format!("ID: {}, URL: {}", redirect.id, redirect.url),
    }
}

/// Decodes a byte buffer into an ID
// @TODO: use webnum
fn decode_id(encoded_id: AsRef<&[u8]>) -> anyhow::Result<i32> {
    Ok(std::str::from_utf8(&base64::decode_config(encoded_id, BASE64_ENCODE_CONFIG)?)?.parse::<i32>()?)
}

/// Encodes an ID to a efficient string format
fn encode_id(encoded_id: AsRef<&[u8]>) -> anyhow::Result<i32> {
    Ok(0)
}

/// Parses a URL and performs validation checks, such that it can be reasonably trusted.
// This definitely isn't tamperproof, but I deem it _good enough._
fn parse_url(url: &str) -> anyhow::Result<Url> {
    let url = match Url::parse(url) {
        Ok(url) => url,
        Err(_e) => Url::parse(&format!("https://{}", url))?,
    };

    // is scheme valid
    if !ALLOWED_URL_SCHEMES.iter().any(|&scheme| scheme == url.scheme()) {
        bail!("URL scheme {} is not a valid scheme", url.scheme());
    }

    let no_domain_error = anyhow!("URL does not have a domain");

    // slight magic ensues
    match url.host_str() {
        None => bail!("No host specified"),
        Some(host) =>  match host.find('.') {
            Some(v) if v == 0 || v == host.len()-1 => bail!("No TLD specified"), // if the period is the first or last character
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
    let routes = routes![
        index,
        show_all_redirects,
        new_redirect,
        redirector,
    ];

    rocket::ignite()
        .mount("/", routes)
        .attach(DbConn::fairing())
        .launch();
}
