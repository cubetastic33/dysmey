use super::UserDetails;
use rand::prelude::*;
use rand_hc::Hc128Rng;
use argon2::{self, Config};
use postgres::Client;
use rocket::{
    http::{Cookie, Cookies},
    request::Form,
};

/*
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email VARCHAR (100) UNIQUE NOT NULL,
    password CHAR (62) NOT NULL
)

CREATE TABLE IF NOT EXISTS trackers (
    id VARCHAR (8) PRIMARY KEY,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    description VARCHAR (600)
)

CREATE TABLE IF NOT EXISTS tracked_visits (
    visit_id SERIAL PRIMARY KEY,
    tracking_id VARCHAR (8) REFERENCES trackers(id) ON DELETE CASCADE,
    time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    ip_address VARCHAR NOT NULL,
    user_agent VARCHAR NOT NULL
)
*/

// Function to check if the email is available
pub fn email_available(client: &mut Client, email: &str) -> bool {
    client.execute("DROP TABLE users", &[]).unwrap();
    client.execute("DROP TABLE tracked_visits", &[]).unwrap();
    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            email VARCHAR (100) UNIQUE NOT NULL,
            password CHAR (62) NOT NULL
        )",
        &[]
    ).unwrap();
    client.execute(
        "CREATE TABLE IF NOT EXISTS trackers (
            id VARCHAR (8) PRIMARY KEY,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            description VARCHAR (600)
        )",
        &[]
    ).unwrap();
    client.execute(
        "CREATE TABLE IF NOT EXISTS tracked_visits (
            visit_id SERIAL PRIMARY KEY,
            tracking_id VARCHAR (8) REFERENCES trackers(id) ON DELETE CASCADE,
            time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            ip_address VARCHAR NOT NULL,
            user_agent VARCHAR NOT NULL
        )",
        &[]
    ).unwrap();
    let rng = thread_rng();
    let salt = Hc128Rng::from_rng(rng).unwrap().next_u32();
    let config = Config::default();
    let password_hash = argon2::hash_encoded(
        String::from("random password").as_bytes(),
        &salt.to_ne_bytes(),
        &config
    ).unwrap();
    println!("hash: {}", password_hash);
    if email.len() == 0 {
        // The email has length 0
        return false;
    }
    // Return whether there are no rows with the given email
    client.query("SELECT * FROM users WHERE email = $1", &[&email])
        .unwrap()
        .is_empty()
}

// Function to create a user with the given details if they're valid
pub fn create_user(
    client: &mut Client,
    user_details: Form<UserDetails>,
    mut cookies: Cookies
) -> String {
    if user_details.email.len() == 0 {
        return String::from("Error: email not provided");
    } else if user_details.password.len() < 8 {
        return String::from("Error: password has to be at least 8 characters");
    }
    // Generate salt using a CSPRNG
    let rng = thread_rng();
    let salt = Hc128Rng::from_rng(rng).unwrap().next_u32();
    let config = Config::default();
    let password_hash = argon2::hash_encoded(
        &user_details.password.as_bytes(),
        &salt.to_ne_bytes(),
        &config
    ).unwrap();
    if let Err(e) = client.query("INSERT INTO users VALUES (
        DEFAULT, $1, $2
    )", &[&user_details.email, &password_hash]) {
        println!("Error: {}", e.to_string());
        return e.to_string();
    }
    cookies.add_private(Cookie::new("email", user_details.email.clone()));
    cookies.add_private(Cookie::new("hash", password_hash));
    return String::from("Success");
}
