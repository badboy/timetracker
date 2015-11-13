use iron::prelude::*;
use iron::status;
use rustc_serialize::json;

pub fn json_error(error: &str) -> IronResult<Response> {
    #[derive(RustcEncodable)]
    struct Error<'a> {
        error: &'a str
    }
    let error = Error { error: error };
    let json = json::encode(&error).unwrap();
    Ok(Response::with((status::Ok, json)))
}

pub fn unauthorized() -> IronResult<Response> {
    Ok(Response::with((status::Unauthorized, "Unauthorized")))
}
pub fn notfound() -> IronResult<Response> {
    Ok(Response::with((status::NotFound, "Not found")))
}
