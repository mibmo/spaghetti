use super::schema::redirects;

#[derive(Queryable)]
pub struct Redirect {
    pub id: String,
    pub url: String,
}

#[derive(Insertable)]
#[table_name="redirects"]
pub struct NewRedirect {
    pub id: String,
    pub url: String,
}
