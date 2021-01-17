use super::schema::redirects;

use rocket::request::FromForm;

#[derive(Queryable)]
pub struct Redirect {
    pub id: i32,
    pub url: String,
}

#[derive(Insertable, FromForm)]
#[table_name="redirects"]
pub struct NewRedirect {
    pub url: String,
}
