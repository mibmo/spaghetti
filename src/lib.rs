#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_contrib;

use diesel::prelude::*;

pub mod schema;
pub mod models;

use models::*;
 
#[database("redirect_db")]
pub struct RedirectDb(diesel::pg::PgConnection);

use schema::redirects;
impl RedirectDb {
    pub fn get_all_redirects(&self) -> QueryResult<Vec<Redirect>> {
        let conn = &self.0;

        redirects::table
            .load::<Redirect>(conn)
    }

    pub fn create_redirect(&self, url: &str) -> QueryResult<i32> {
        let conn = &self.0;

        let new_redirect = NewRedirect {
            url: url.to_string(),
        };

        diesel::insert_into(redirects::table)
            .values(&new_redirect)
            .returning(redirects::dsl::id)
            .get_result(conn)
    }

    pub fn get_redirect_with_id(&self, id: i32) -> QueryResult<Redirect> {
        let conn = &self.0;

        redirects::table
            .filter(redirects::dsl::id.eq(id))
            .first::<Redirect>(conn)
    }
}
