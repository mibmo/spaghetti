#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_contrib;

use rand::{prelude::*, Rng, distributions::Alphanumeric};
use diesel::prelude::*;

pub mod schema;
pub mod models;

use models::*;

const ID_LENGTH: usize = 5;
 
#[database("redirect_db")]
pub struct RedirectDb(diesel::pg::PgConnection);

use schema::redirects;
impl RedirectDb {
    pub fn get_all_redirects(&self) -> QueryResult<Vec<Redirect>> {
        let conn = &self.0;

        redirects::table
            .load::<Redirect>(conn)
    }

    pub fn create_redirect(&self, url: &str) -> QueryResult<String> {
        let conn = &self.0;

        // inefficient. has to wait for system entropy as well
        // consider using nanoid crate
        let rng = SmallRng::from_entropy(); 
        let random_id: String = rng
            .sample_iter(&Alphanumeric)
            .take(ID_LENGTH)
            .map(char::from)
            .collect();

        let new_redirect = NewRedirect {
            id: random_id,
            url: url.to_string(),
        };

        diesel::insert_into(redirects::table)
            .values(&new_redirect)
            .returning(redirects::dsl::id)
            .get_result(conn)
    }

    pub fn get_redirect_with_id(&self, id: &str) -> QueryResult<Redirect> {
        let conn = &self.0;

        redirects::table
            .filter(redirects::dsl::id.eq(id))
            .first::<Redirect>(conn)
    }
}
