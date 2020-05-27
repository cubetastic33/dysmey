use super::{UserDetails, Context};
use argon2::{self, Config};
use postgres::Client;
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
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    description VARCHAR (300)
)

CREATE TABLE IF NOT EXISTS tracked_visits (
    visit_id SERIAL PRIMARY KEY,
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
            if argon2::verify_encoded(
                &password_hash,
                user_details.password.as_bytes()
            ).unwrap() {
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
    let rows = client.query("SELECT FROM users WHERE email = $1 AND password_hash = $2", &[&email, &hash]);
    !rows.unwrap().is_empty()
}

impl Context {
    pub fn new(client: &mut Client, mut cookies: Cookies) -> Self {
        client.execute("DROP TABLE tracked_visits", &[]).unwrap();
        client.execute("DROP TABLE trackers", &[]).unwrap();
        client.execute("CREATE TABLE IF NOT EXISTS trackers (
            id VARCHAR (8) PRIMARY KEY,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            user_email VARCHAR NOT NULL REFERENCES users(email) ON DELETE CASCADE,
            description VARCHAR (300)
        )", &[]).unwrap();
        client.execute("CREATE TABLE IF NOT EXISTS tracked_visits (
            visit_id SERIAL PRIMARY KEY,
            tracking_id VARCHAR (8) REFERENCES trackers(id) ON DELETE CASCADE,
            time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            ip_address VARCHAR NOT NULL,
            user_agent VARCHAR NOT NULL
        )", &[]).unwrap();
        if let Some(email) = cookies.get_private("email") {
            if let Some(hash) = cookies.get_private("hash") {
                // If the email and hash cookies are present
                if verify_credentials(client, email.value(), hash.value()) {
                    // If the credentials are correct
                    return Context {
                        email: Some(email.value().to_owned()),
                        photo: Some(format!("https://www.gravatar.com/avatar/{:x}?d=retro", md5::compute(email.value().to_lowercase()))),
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
            for _tracker in client
                .query(
                    "SELECT * FROM trackers WHERE user_email = $1",
                    &[email],
                )
                .unwrap() {
                self.trackers = vec![];
            }
        }
    }
}
