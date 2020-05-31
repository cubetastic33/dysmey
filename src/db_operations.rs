use super::{Context, AdminDetails, TrackerInfo, RequestDetails, Tracker, TrackerRequest, UserDetails};
use argon2::{self, Config};
use postgres::Client;
use chrono::NaiveDateTime;
use rand::prelude::*;
use rand_hc::Hc128Rng;
use rocket::{
    http::{Cookie, Cookies},
    request::Form,
};

/*
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email VARCHAR (100) UNIQUE NOT NULL,
    password_hash VARCHAR NOT NULL
)

CREATE TABLE IF NOT EXISTS trackers (
    id VARCHAR (8) PRIMARY KEY,
    user_email VARCHAR NOT NULL REFERENCES users(email) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    description VARCHAR (300)
)

CREATE TABLE IF NOT EXISTS tracked_requests (
    request_id SERIAL PRIMARY KEY,
    tracking_id VARCHAR (8) REFERENCES trackers(id) ON DELETE CASCADE,
    time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    ip_address VARCHAR NOT NULL,
    user_agent VARCHAR NOT NULL
)
*/

// Function to create a user with the given details if they're valid
pub fn signup_user(
    client: &mut Client,
    user_details: Form<UserDetails>,
    mut cookies: Cookies,
) -> String {
    if user_details.email.len() == 0 {
        return String::from("Error: email not provided");
    } else if user_details.password.len() < 8 {
        return String::from("Error: password has to be at least 8 characters");
    }
    // Generate salt using a CSPRNG
    let rng = thread_rng();
    let salt = Hc128Rng::from_rng(rng).unwrap().next_u64();
    let config = Config::default();
    let password_hash = argon2::hash_encoded(
        &user_details.password.as_bytes(),
        &salt.to_ne_bytes(),
        &config,
    )
    .unwrap();
    if let Err(e) = client.query(
        "INSERT INTO users VALUES (
        DEFAULT, $1, $2
    )",
        &[&user_details.email, &password_hash],
    ) {
        println!("Error: {}", e.to_string());
        return e.to_string();
    }
    cookies.add_private(Cookie::new("email", user_details.email.clone()));
    cookies.add_private(Cookie::new("hash", password_hash));
    return String::from("Success");
}

// Function to sign in the user if the given credentials are correct
pub fn signin_user(
    client: &mut Client,
    user_details: Form<UserDetails>,
    mut cookies: Cookies,
) -> String {
    if user_details.email.len() > 0 && user_details.password.len() >= 8 {
        // Proceed if email and password have been provided
        let rows = client
            .query(
                "SELECT password_hash FROM users WHERE email = $1",
                &[&user_details.email],
            )
            .unwrap();
        if !rows.is_empty() {
            // Compare passwords if email exists
            let password_hash: String = rows.get(0).unwrap().get(0);
            if argon2::verify_encoded(&password_hash, user_details.password.as_bytes()).unwrap() {
                // If the credentials are correct, create the cookies to sign the user in
                cookies.add_private(Cookie::new("email", user_details.email.clone()));
                cookies.add_private(Cookie::new("hash", password_hash));
                return String::from("Success");
            }
        }
    }
    return String::from("Invalid credentials");
}

// Function to sign a user out
pub fn signout_user(mut cookies: Cookies) -> String {
    cookies.remove_private(Cookie::named("email"));
    cookies.remove_private(Cookie::named("hash"));
    String::from("Success")
}

// Function to verify if the given credentials are correct
fn verify_credentials(client: &mut Client, email: &str, hash: &str) -> bool {
    let rows = client.query(
        "SELECT FROM users WHERE email = $1 AND password_hash = $2",
        &[&email, &hash],
    );
    !rows.unwrap().is_empty()
}

// Function to get a new tracking ID
pub fn new_tracking_id(client: &mut Client) -> String {
    let mut rng = thread_rng();
    let mut tracking_id = format!("{:x}", rng.gen_range(0x10000000, 0xFFFFFFFF_u32));
    let mut rows = client
        .query("SELECT * FROM trackers WHERE id = $1", &[&tracking_id])
        .unwrap();
    while !rows.is_empty() {
        tracking_id = format!("{:x}", rng.gen_range(0x10000000, 0xFFFFFFFF_u32));
        rows = client
            .query("SELECT * FROM trackers WHERE id = $1", &[&tracking_id])
            .unwrap();
    }
    tracking_id
}

// Function to register a tracker
pub fn register_tracker(
    client: &mut Client,
    new_tracker: Form<TrackerInfo>,
    mut cookies: Cookies,
) -> String {
    if let Some(email) = cookies.get_private("email") {
        if let Some(hash) = cookies.get_private("hash") {
            // If the email and hash cookies are present
            if verify_credentials(client, email.value(), hash.value()) {
                // If the credentials are correct
                if !client
                    .query(
                        "SELECT * FROM trackers WHERE id = $1",
                        &[&new_tracker.tracking_id],
                    )
                    .unwrap()
                    .is_empty() {
                    return format!("Tracking ID {} already in use", new_tracker.tracking_id);
                }
                client
                    .execute(
                        "INSERT INTO trackers VALUES ($1, $2, DEFAULT, $3)",
                        &[
                            &new_tracker.tracking_id,
                            &email.value(),
                            &new_tracker.description,
                        ],
                    )
                    .unwrap();
                return String::from("Success");
            }
        }
    }
    String::from("Not signed in")
}

// Function to update the description of a tracker
pub fn update_description(
    client: &mut Client,
    tracker_info: Form<TrackerInfo>,
    mut cookies: Cookies,
) -> String {
    if let Some(email) = cookies.get_private("email") {
        if let Some(hash) = cookies.get_private("hash") {
            // If the email and hash cookies are present
            if verify_credentials(client, email.value(), hash.value()) {
                // If the credentials are correct
                if client
                    .query(
                        "SELECT * FROM trackers WHERE id = $1 AND user_email = $2",
                        &[&tracker_info.tracking_id, &email.value()],
                    )
                    .unwrap()
                    .is_empty() {
                    return format!("Tracking ID {} not found under user", tracker_info.tracking_id);
                }
                client
                    .execute(
                        "UPDATE trackers SET description = $1 WHERE id = $2",
                        &[&tracker_info.description, &tracker_info.tracking_id],
                    )
                    .unwrap();
                return String::from("Success");
            }
        }
    }
    String::from("Not signed in")
}

// Function to delete a tracker
pub fn delete_tracker(client: &mut Client, tracking_id: &str, mut cookies: Cookies) -> String {
    if let Some(email) = cookies.get_private("email") {
        if let Some(hash) = cookies.get_private("hash") {
            // If the email and hash cookies are present
            if verify_credentials(client, email.value(), hash.value()) {
                // If the credentials are correct
                if client
                    .query(
                        "SELECT * FROM trackers WHERE id = $1 AND user_email = $2",
                        &[&tracking_id, &email.value()],
                    )
                    .unwrap()
                    .is_empty() {
                    return format!("Tracking ID {} not found under user", tracking_id);
                }
                client
                    .execute(
                        "DELETE FROM trackers WHERE id = $1",
                        &[&tracking_id],
                    )
                    .unwrap();
                return String::from("Success");
            }
        }
    }
    String::from("Not signed in")
}

// Function to delete a tracker
pub fn delete_request(client: &mut Client, request_id: i32, mut cookies: Cookies) -> String {
    if let Some(email) = cookies.get_private("email") {
        if let Some(hash) = cookies.get_private("hash") {
            // If the email and hash cookies are present
            if verify_credentials(client, email.value(), hash.value()) {
                // The credentials are correct
                // Check if the request exists, and get the tracker ID if it does
                if let Some(tracking_id) = client
                    .query_opt(
                        "SELECT tracking_id FROM tracked_requests WHERE request_id = $1",
                        &[&request_id],
                    )
                    .unwrap() {
                    // The request exists; make sure the tracker belongs to the logged in user
                    if !client
                        .query(
                            "SELECT * FROM trackers WHERE id = $1 AND user_email = $2",
                            &[&tracking_id.get::<_, String>(0), &email.value()],
                        )
                        .unwrap()
                        .is_empty() {
                        // It does belong to the logged in user
                        // Delete the request
                        client
                            .execute(
                                "DELETE FROM tracked_requests WHERE request_id = $1",
                                &[&request_id],
                                )
                            .unwrap();
                        return String::from("Success");
                    }
                }
                // The request either didn't exist or belong to the logged in user
                return format!("Request ID {} not found under user", request_id);
            }
        }
    }
    String::from("Not signed in")
}

// Function to save a request if it's being tracked
pub fn save_request(client: &mut Client, tracking_id: String, request_details: RequestDetails) {
    if let Some(_) = client
        .query_opt("SELECT * FROM trackers WHERE id = $1", &[&tracking_id])
        .unwrap() {
        client
            .execute(
                "INSERT INTO tracked_requests VALUES (DEFAULT, $1, DEFAULT, $2, $3)",
                &[
                    &tracking_id,
                    &request_details.ip_address,
                    &request_details.user_agent,
                ],
            )
            .unwrap();
    }
}

impl Context {
    pub fn new(client: &mut Client, mut cookies: Cookies) -> Self {
        if let Some(email) = cookies.get_private("email") {
            if let Some(hash) = cookies.get_private("hash") {
                // If the email and hash cookies are present
                if verify_credentials(client, email.value(), hash.value()) {
                    // If the credentials are correct
                    return Context {
                        email: Some(email.value().to_owned()),
                        photo: Some(format!(
                            "https://www.gravatar.com/avatar/{:x}?d=retro",
                            md5::compute(email.value().to_lowercase())
                        )),
                        trackers: Vec::new(),
                    };
                }
            }
        }
        // The credentials were wrong/absent
        Context {
            email: None,
            photo: None,
            trackers: Vec::new(),
        }
    }

    // Function to populate the trackers field if signed in
    pub fn get_trackers(&mut self, client: &mut Client) {
        if let Some(email) = &self.email {
            for tracker_row in client
                .query(
                    "SELECT * FROM trackers WHERE user_email = $1 ORDER BY created_at DESC",
                    &[email],
                )
                .unwrap() {
                let mut tracker = Tracker {
                    tracking_id: tracker_row.get(0),
                    created_at: tracker_row.get::<_, NaiveDateTime>(2).timestamp(),
                    description: tracker_row.get(3),
                    requests: Vec::new(),
                };
                for tracked_request in client
                    .query(
                        "SELECT * FROM tracked_requests WHERE tracking_id = $1 ORDER BY time DESC",
                        &[&tracker.tracking_id],
                    )
                    .unwrap() {
                    let time: NaiveDateTime = tracked_request.get(2);
                    tracker.requests.push(TrackerRequest {
                        id: tracked_request.get(0),
                        time: time.timestamp(),
                        ip_address: tracked_request.get(3),
                        user_agent: tracked_request.get(4),
                    });
                }
                self.trackers.push(tracker);
            }
        }
    }
}

impl AdminDetails {
    pub fn new(client: &mut Client, mut cookies: Cookies) -> Self {
        let mut photo = None;
        if let Some(email) = cookies.get_private("email") {
            if let Some(hash) = cookies.get_private("hash") {
                // If the email and hash cookies are present
                if verify_credentials(client, email.value(), hash.value()) {
                    // If the credentials are correct
                    photo = Some(format!(
                        "https://www.gravatar.com/avatar/{:x}?d=retro",
                        md5::compute(email.value().to_lowercase())
                    ));
                }
            }
        }
        let mut admin_details = AdminDetails {
            photo,
            ..Default::default()
        };
        for user_row in client.query("SELECT user_email FROM users ORDER BY id DESC", &[]).unwrap() {
            let mut user = (user_row.get(0), 0, 0);
            for tracker in client.query("SELECT id, created_at FROM trackers WHERE user_email = $1", &[&user_row.get::<_, String>(0)]).unwrap() {
                user.1 += 1;
                admin_details.trackers.push(tracker.get::<_, NaiveDateTime>(1).timestamp());
                for request in client.query("SELECT time FROM tracked_requests WHERE tracking_id = $1", &[&tracker.get::<_, String>(0)]).unwrap() {
                    user.2 += 1;
                    admin_details.requests.push(request.get::<_, NaiveDateTime>(0).timestamp());
                }
            }
            admin_details.users.push(user);
        }
        admin_details
    }
}
