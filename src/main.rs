#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use postgres::{Client, NoTls};
use rocket::{
    http::Cookies,
    request::{self, Form, FromRequest, Request},
    response::{self, NamedFile, Redirect, Responder},
    Config, Outcome, Response, State,
};
use rocket_contrib::{serve::StaticFiles, templates::Template};
use std::{env, sync::Mutex};

mod db_operations;

use db_operations::*;

#[derive(Serialize)]
struct TrackerRequest {
    time: String,
    ip_address: String,
    user_agent: String,
}

#[derive(Serialize)]
struct Tracker {
    tracking_id: String,
    created_at: String,
    description: String,
    requests: Vec<TrackerRequest>,
}

#[derive(Serialize)]
struct Context {
    email: Option<String>,
    photo: Option<String>,
    trackers: Vec<Tracker>,
}

#[derive(FromForm)]
pub struct UserDetails {
    email: String,
    password: String,
}

#[derive(FromForm)]
pub struct NewTracker {
    tracking_id: String,
    description: String,
}

#[derive(Debug)]
pub struct RequestDetails {
    ip_address: String,
    user_agent: String,
}

struct EmptyImage {}

#[derive(Debug)]
pub enum RequestDetailsError {}

impl<'r> Responder<'r> for EmptyImage {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(NamedFile::open("empty.png").unwrap().respond_to(req)?)
            .raw_header("Cache-Control", "no-cache")
            .ok()
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for RequestDetails {
    type Error = RequestDetailsError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let ip_address = match request.headers().get("X-Forwarded-For").next() {
            Some(ip) => ip.to_string(),
            None => String::from("missing"),
        };
        Outcome::Success(RequestDetails {
            ip_address,
            user_agent: request
                .headers()
                .get("User-Agent")
                .next()
                .unwrap()
                .to_string(),
        })
    }
}

#[get("/")]
fn get_index(client: State<Mutex<Client>>, cookies: Cookies) -> Template {
    Template::render("index", Context::new(&mut client.lock().unwrap(), cookies))
}

#[get("/signin")]
fn get_signin(client: State<Mutex<Client>>, cookies: Cookies) -> Template {
    Template::render("signin", Context::new(&mut client.lock().unwrap(), cookies))
}

#[get("/signup")]
fn get_signup(client: State<Mutex<Client>>, cookies: Cookies) -> Template {
    Template::render("signup", Context::new(&mut client.lock().unwrap(), cookies))
}

#[get("/dashboard")]
fn get_dashboard(client: State<Mutex<Client>>, cookies: Cookies) -> Result<Template, Redirect> {
    let mut context = Context::new(&mut client.lock().unwrap(), cookies);
    if context.email != None {
        context.get_trackers(&mut client.lock().unwrap());
        Ok(Template::render("dashboard", context))
    } else {
        Err(Redirect::to("/signin"))
    }
}

#[get("/track/<tracking_id>")]
fn get_track(
    client: State<Mutex<Client>>,
    request_details: RequestDetails,
    tracking_id: String,
) -> EmptyImage {
    save_request(&mut client.lock().unwrap(), tracking_id, request_details);
    EmptyImage {}
}

#[post("/signin", data = "<user_details>")]
fn post_signin(
    client: State<Mutex<Client>>,
    user_details: Form<UserDetails>,
    cookies: Cookies,
) -> String {
    signin_user(&mut client.lock().unwrap(), user_details, cookies)
}

#[post("/signup", data = "<user_details>")]
fn post_signup(
    client: State<Mutex<Client>>,
    user_details: Form<UserDetails>,
    cookies: Cookies,
) -> String {
    signup_user(&mut client.lock().unwrap(), user_details, cookies)
}

#[post("/signout")]
fn post_signout(cookies: Cookies) -> String {
    signout_user(cookies)
}

#[post("/new_tracking_id")]
fn post_new_tracking_id(client: State<Mutex<Client>>) -> String {
    new_tracking_id(&mut client.lock().unwrap())
}

#[post("/register_tracker", data = "<new_tracker>")]
fn post_register_tracker(
    client: State<Mutex<Client>>,
    new_tracker: Form<NewTracker>,
    cookies: Cookies,
) -> String {
    register_tracker(&mut client.lock().unwrap(), new_tracker, cookies)
}

fn configure() -> Config {
    // Configure Rocket to serve on the port requested by Heroku.
    let mut config = Config::active().expect("could not load configuration");
    config
        .set_secret_key(env::var("SECRET_KEY").unwrap())
        .unwrap();
    let port = if let Ok(port_str) = env::var("PORT") {
        port_str.parse().expect("could not parse PORT")
    } else {
        7733
    };
    config.set_port(port);
    config
}

fn rocket() -> rocket::Rocket {
    rocket::custom(configure())
        .mount(
            "/",
            routes![
                get_index,
                get_signin,
                get_signup,
                get_dashboard,
                get_track,
                post_signin,
                post_signup,
                post_signout,
                post_new_tracking_id,
                post_register_tracker,
            ],
        )
        .mount("/styles", StaticFiles::from("static/styles"))
        .mount("/scripts", StaticFiles::from("static/scripts"))
        .mount("/fonts", StaticFiles::from("static/fonts"))
        .mount("/images", StaticFiles::from("static/images"))
        .attach(Template::fairing())
}

fn main() {
    let client = Client::connect(&env::var("DATABASE_URL").unwrap(), NoTls).unwrap();
    rocket().manage(Mutex::new(client)).launch();
}
