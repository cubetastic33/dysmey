use super::UserDetails;
use rand::prelude::*;
use rand_hc::Hc128Rng;
use postgres::Client;
use rocket::{
    http::{Cookie, Cookies},
    request::Form,
};

// Function to check if the email is available
pub fn email_available(client: &mut Client, email: &str) -> bool {
    client.execute(
        "CREATE TABLE IF NOT EXISTS tracked_visits (
            id PRIMARY KEY,
            time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            ip_address VARCHAR NOT NULL,
            user_agent VARCHAR NOT NULL
        )",
        &[]
    ).unwrap();
    let rng = thread_rng();
    let salt = rand_hc::Hc128Rng::from_rng(rng).unwrap().next_u32();
    println!("salt 1: {}", salt);
    let salt = rand_hc::Hc128Rng::from_rng(rng).unwrap().next_u32();
    println!("salt 2: {}", salt);
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
    let mut rng = thread_rng();
    let salt = rand_hc::Hc128Rng::from_rng(rng).unwrap().next_u32();
    return String::from("");
}
