#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use dotenv::dotenv;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use postgres::Client;
use rocket::{
    http::{Cookie, Cookies},
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
    id: i32,
    time: i64,
    ip_address: String,
    user_agent: String,
}

#[derive(Serialize)]
struct Tracker {
    tracking_id: String,
    created_at: i64,
    description: String,
    requests: Vec<TrackerRequest>,
}

#[derive(Serialize)]
struct Context {
    email: Option<String>,
    photo: Option<String>,
    trackers: Vec<Tracker>,
}

#[derive(Serialize)]
struct AdminDetails {
    photo: Option<String>,
    users: Vec<(String, usize, usize)>,
    trackers: Vec<i64>,
    requests: Vec<i64>,
}

#[derive(FromForm)]
pub struct UserDetails {
    email: String,
    password: String,
}

#[derive(FromForm)]
pub struct TrackerInfo {
    tracking_id: String,
    description: String,
}

#[derive(FromForm)]
pub struct SingleField {
    value: String,
}

#[derive(Debug)]
pub struct RequestDetails {
    ip_address: String,
    user_agent: String,
}

struct EmptyImage {}

#[derive(Debug)]
pub enum RequestDetailsError {}

impl Default for AdminDetails {
    fn default() -> Self {
        AdminDetails {
            photo: None,
            users: Vec::new(),
            trackers: Vec::new(),
            requests: Vec::new(),
        }
    }
}

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

#[get("/admin")]
fn get_admin(client: State<Mutex<Client>>, mut cookies: Cookies) -> Template {
    if let Some(password) = cookies.get_private("admin_password") {
        if password.value() == env::var("ADMIN_PASSWORD").unwrap() {
            return Template::render("admin", AdminDetails::new(&mut client.lock().unwrap(), cookies));
        }
    }
    Template::render("admin_signin", AdminDetails::default())
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

#[post("/register_tracker", data = "<tracker_info>")]
fn post_register_tracker(
    client: State<Mutex<Client>>,
    tracker_info: Form<TrackerInfo>,
    cookies: Cookies
) -> String {
    register_tracker(&mut client.lock().unwrap(), tracker_info, cookies)
}

#[post("/update_description", data = "<tracker_info>")]
fn post_update_description(
    client: State<Mutex<Client>>,
    tracker_info: Form<TrackerInfo>,
    cookies: Cookies
) -> String {
    update_description(&mut client.lock().unwrap(), tracker_info, cookies)
}

#[post("/delete_tracker", data = "<tracking_id>")]
fn post_delete_tracker(
    client: State<Mutex<Client>>,
    tracking_id: Form<SingleField>,
    cookies: Cookies
) -> String {
    delete_tracker(&mut client.lock().unwrap(), &tracking_id.value, cookies)
}

#[post("/delete_request", data = "<request_id>")]
fn post_delete_request(
    client: State<Mutex<Client>>,
    request_id: Form<SingleField>,
    cookies: Cookies
) -> String {
    delete_request(&mut client.lock().unwrap(), request_id.value.parse::<i32>().unwrap(), cookies)
}

#[post("/admin_signin", data = "<password>")]
fn post_admin_signin(password: Form<SingleField>, mut cookies: Cookies) -> String {
    if password.value == env::var("ADMIN_PASSWORD").unwrap() {
        cookies.add_private(Cookie::new("admin_password", password.value.clone()));
        String::from("Success")
    } else {
        String::from("Incorrect password")
    }
}

#[post("/signout_admin")]
fn post_signout_admin(cookies: Cookies) -> String {
    signout_admin(cookies)
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
                get_admin,
                post_signin,
                post_signup,
                post_signout,
                post_new_tracking_id,
                post_register_tracker,
                post_update_description,
                post_delete_tracker,
                post_delete_request,
                post_admin_signin,
                post_signout_admin,
            ],
        )
        .mount("/styles", StaticFiles::from("static/styles"))
        .mount("/scripts", StaticFiles::from("static/scripts"))
        .mount("/fonts", StaticFiles::from("static/fonts"))
        .mount("/images", StaticFiles::from("static/images"))
        .mount("/videos", StaticFiles::from("static/videos"))
        .mount("/", StaticFiles::from("static/icons").rank(20))
        .attach(Template::fairing())
}

fn main() {
    dotenv().ok();
    let connector = MakeTlsConnector::new(TlsConnector::builder().danger_accept_invalid_certs(true).build().unwrap());

    let client = Client::connect(&env::var("DATABASE_URL").unwrap(), connector).unwrap();
    rocket().manage(Mutex::new(client)).launch();
}
