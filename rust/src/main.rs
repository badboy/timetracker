extern crate iron;
extern crate persistent;
extern crate cookie;
extern crate oatmeal_raisin;
#[macro_use]
extern crate router;
extern crate staticfile;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate ohmers;
extern crate rustc_serialize;

extern crate redis;
extern crate r2d2;
extern crate r2d2_redis;
extern crate rand;
extern crate urlencoded;

use iron::prelude::*;
use iron::status;
use iron::headers::{AccessControlAllowOrigin, Location};

use iron::typemap::Key;
use router::Router;
use persistent::Read;
use oatmeal_raisin::{SetCookie, SigningKey};
use urlencoded::UrlEncodedBody;

use std::path::Path;
use staticfile::Static;

use std::ops::Deref;
use r2d2::Pool;
use r2d2_redis::{RedisConnectionManager};
use redis::Commands;
use ohmers::{Ohmer, Reference, with, get};
use rustc_serialize::json;

use std::str::FromStr;
use std::env;

mod models;
mod user_handling;
mod responses;
use models::{User, TimeTrack, TimeTrackView};
use user_handling::UserFetch;
use responses::*;

pub type RedisPool = Pool<RedisConnectionManager>;
pub struct AppDb;
impl Key for AppDb { type Value = RedisPool; }

fn main() {
    env_logger::init().unwrap();
    info!("Starting server");

    let config = r2d2::Config::builder()
        .connection_timeout_ms(2*1000)
        .pool_size(3)
        .build();

    let redis_url = env::var("REDIS_URL")
        .unwrap_or("redis://localhost".into());
    let static_path = env::var("STATIC_PATH")
        .unwrap_or("../frontend".into());

    let manager = RedisConnectionManager::new(&redis_url[..]).unwrap();
    let pool = r2d2::Pool::new(config, manager).unwrap();

    let router = router!(post "/api/time/new" => new_track,
                         get "/api/time/:id" => show_track,
                         get "/api/time" => show_all_tracks,
                         get "/" => redirect_to_index,
                         get "/*" => Static::new(Path::new(&static_path))
                        );

    let mut chain = Chain::new(router);

    chain.link_before(|req: &mut Request| {
        // Basic logging of requests.
        info!("REQUEST: {}", req.url.path.join("/"));
        Ok(())
    });

    // Signing key for cookies
    chain.link_before(Read::<SigningKey>::one(b"ba8742af4750"));
    // The Redis Connection pool
    chain.link_before(Read::<AppDb>::one(pool));
    // The user handling before every request
    chain.link(UserFetch::both());
    // Make sure all cookies are set before sending the response
    chain.link_after(SetCookie);

    chain.link_after(|_: &mut Request, mut res: Response| {
        // lol
        res.headers.set(AccessControlAllowOrigin::Any);
        Ok(res)
    });

    info!("Server starting on http://localhost:3000");
    Iron::new(chain).http("localhost:3000").unwrap();
}

fn redirect_to_index(_: &mut Request) -> IronResult<Response> {
    let mut res = Response::with(status::TemporaryRedirect);
    res.headers.set(Location("index.html".into()));
    Ok(res)
}

/// POST /api/time/new
///
/// Parameters:
///
/// * `start`: The start timestamp, MUST be larger than 0
/// * `stop`: The stop timestamp, MUST be larger than 0
///
/// Saves a new time tracker for the current user
/// and returns it.
fn new_track(req: &mut Request) -> IronResult<Response> {
    let ref pool = req.get::<Read<AppDb>>().unwrap();
    let conn = pool.get().unwrap();
    let user = {
        let user = req.extensions.get::<User>().unwrap();
        user.clone()
    };

    match req.get_ref::<UrlEncodedBody>() {
        Err(_) => return notfound(),
        Ok(hashmap) => {
            info!("Parsed body: {:?}", hashmap);

            let start = hashmap.get("start")
                .map(|s| u64::from_str(&s[0]).unwrap_or(0))
                .unwrap_or(0);
            let stop = hashmap.get("stop")
                .map(|s| u64::from_str(&s[0]).unwrap_or(0))
                .unwrap_or(0);

            info!("Start: {:?}, Stop: {:?}", start, stop);
            if start == 0 || stop == 0 {
                return json_error("Start and stop parameters required.");
            }

            let track = create!(TimeTrack{
                start: start,
                stop: stop,
                user: Reference::with_value(&user)
            }, *conn.deref()).unwrap();
            info!("Saving track: {:?}, with user {:?}", track, user);

            let json = json::encode(&TimeTrackView::from(&track)).unwrap();
            Ok(Response::with((status::Ok, json)))
        }
    }
}

/// GET /api/time/:id
///
/// Show a certain time tracker by id for the current user.
/// Throws an error if the requested tracker does not belong
/// to the current user.
fn show_track(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<Read<AppDb>>().unwrap();
    let conn = pool.get().unwrap();
    let user = req.extensions.get::<User>().unwrap();
    let id = req.extensions.get::<Router>().unwrap().find("id").unwrap_or("");
    let id = usize::from_str(id).unwrap_or(0);

    info!("show_track. id={}", id);

    let track : TimeTrack = match get(id, conn.deref()) {
        Err(_) => return notfound(),
        Ok(track) => track
    };

    info!("show_track. user={:?}, track={:?}", user, track);
    let track_user = track.user.get(conn.deref()).unwrap();

    if user.id != track_user.id {
        return unauthorized();
    }

    let track = TimeTrackView::from(&track);
    let encoded = json::encode(&track).unwrap();
    Ok(Response::with((status::Ok, encoded)))
}

/// GET /api/time
///
/// Show all saved time trackers for the current user
fn show_all_tracks(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<Read<AppDb>>().unwrap();
    let conn = pool.get().unwrap();
    let user = req.extensions.get::<User>().unwrap();

    let user = user.clone();
    let tracks = collection!(user.tracks, *conn.deref())
        .try_into_iter()
        .unwrap()
        .map(|t| TimeTrackView::from(&t))
        .collect::<Vec<_>>();

    info!("tracks: {:?}", tracks);

    let encoded = json::encode(&tracks).unwrap();
    Ok(Response::with((status::Ok, encoded)))
}
