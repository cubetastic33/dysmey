#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use chrono::prelude::*;
use postgres::{Client, NoTls};
use rocket::{
    http::Cookies,
    request::{self, Form, FromRequest, Request},
    response::{self, NamedFile, Responder, Redirect},
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

struct EmptyImage {}

#[derive(Debug)]
enum IpAddr {
    IpAddress(String),
    Missing,
}

#[derive(Debug)]
enum IpAddrError {}

impl<'r> Responder<'r> for EmptyImage {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(NamedFile::open("empty.png").unwrap().respond_to(req)?)
            .raw_header("Cache-Control", "no-cache")
            .ok()
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for IpAddr {
    type Error = IpAddrError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        match request.headers().get("X-Forwarded-For").next() {
            Some(ip) => Outcome::Success(IpAddr::IpAddress(ip.to_string())),
            None => Outcome::Success(IpAddr::Missing),
        }
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

#[get("/profile")]
fn get_profile(client: State<Mutex<Client>>, cookies: Cookies) -> Result<Template, Redirect> {
    let mut context = Context::new(&mut client.lock().unwrap(), cookies);
    if context.email != None {
        context.get_trackers(&mut client.lock().unwrap());
        Ok(Template::render("profile", context))
    } else {
        Err(Redirect::to("/signin"))
    }
}

#[get("/track/<tracking_id>")]
fn get_track(ip_address: IpAddr, tracking_id: String) -> EmptyImage {
    if let IpAddr::IpAddress(ip_address) = ip_address {
        println!(
            "\nTime: {}, IP Address: {:?}, Tracking ID: {}",
            Utc::now()
                .with_timezone(&FixedOffset::east(19800))
                .format("%Y-%m-%d %H:%M:%S"),
            ip_address,
            tracking_id
        );
    } else {
        println!(
            "\nTime: {}, Unknown IP Address, Tracking ID: {}",
            Utc::now()
                .with_timezone(&FixedOffset::east(19800))
                .format("%Y-%m-%d %H:%M:%S"),
            tracking_id
        );
    }
    EmptyImage {}
}

#[post("/signin", data = "<user_details>")]
fn post_signin(
    client: State<Mutex<Client>>,
    user_details: Form<UserDetails>,
    cookies: Cookies
) -> String {
    signin_user(&mut client.lock().unwrap(), user_details, cookies)
}

#[post("/signup", data = "<user_details>")]
fn post_signup(
    client: State<Mutex<Client>>,
    user_details: Form<UserDetails>,
    cookies: Cookies
) -> String {
    signup_user(&mut client.lock().unwrap(), user_details, cookies)
}

#[post("/signout")]
fn post_signout(cookies: Cookies) -> String {
    signout_user(cookies)
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
                get_profile,
                get_track,
                post_signin,
                post_signup,
                post_signout,
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
