use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
use iron::middleware;

use std::ops::Deref;
use std::error::Error;
use std::fmt::{self, Debug};

use redis;
use cookie;
use persistent::Read;
use oatmeal_raisin::{Cookie, CookieJar};
use ohmers::{Ohmer, OhmerError, with};
use rand::{thread_rng, Rng};

use models::User;
use super::AppDb;

pub struct UserFetch;

impl UserFetch {
    pub fn both() -> (UserFetch,UserFetch) {
        (UserFetch, UserFetch)
    }
}

impl Key for User { type Value = User; }

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str { &*self.0 }
}

impl middleware::BeforeMiddleware for UserFetch {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let pool = req.get::<Read<AppDb>>().unwrap();
        let conn = pool.get().unwrap();

        let user = {
            let cookie_jar = req.get_mut::<CookieJar>().unwrap();
            let sig_jar = cookie_jar.signed();
            fetch_user_or_create(&sig_jar, conn.deref())
        };

        match user {
            None => {
                Err(IronError::new(
                        StringError("Unauthorized".into()),
                        status::Unauthorized))
            }
            Some(user) => {
                req.extensions.insert::<User>(user);
                Ok(())
            }
        }

    }
}

impl middleware::AfterMiddleware for UserFetch {
   fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
       let user = match req.extensions.get::<User>() {
           Some(user) => user.name.clone(),
           None => return Ok(res)
       };

       let cookie_jar = req.get_mut::<CookieJar>().unwrap();
       let sig_jar = cookie_jar.signed();
       sig_jar.add(Cookie::new("user-id".into(), user));
       Ok(res)
   }
}

fn fetch_user(name: String, conn: &redis::Connection) -> Option<User> {
    info!("fetching user with {:?}", name);
    let user : User = match with("name", name, conn) {
        Err(_) => return None,
        Ok(None) => return None,
        Ok(Some(u)) => u
    };

    info!("fetch_user {:?}", user);
    Some(user)
}

fn new_random_user(conn: &redis::Connection) -> Option<User> {
    let mut retries = 5;
    while retries > 0 {
        let name: String = thread_rng().gen_ascii_chars().take(10).collect();
        match create!(User { name: name, }, *conn) {
            Ok(user) => return Some(user),
            Err(OhmerError::UniqueIndexViolation(_)) => {
                retries -= 1;
                continue;
            },
            _ => return None
        };

    }

    None
}

fn fetch_user_or_create(jar: &cookie::CookieJar, conn: &redis::Connection) -> Option<User> {
    info!("fetch_user_or_create");
    let user = match jar.find("user-id") {
        None => return new_random_user(conn),
        Some(cookie) => {
            let name = cookie.value;
            fetch_user(name, conn)
        }
    };

    match user {
        None => new_random_user(conn),
        Some(user) => Some(user)
    }
}

