use postgres::Client;

// Function to check if the email is available
pub fn email_available(client: &mut Client, email: &str) -> bool {
    if email.len() == 0 {
        // The email has length 0
        return false;
    }
    // Return whether there are no rows with the given email
    client.query("SELECT * FROM users WHERE email = $1", &[&email])
        .unwrap()
        .is_empty()
}
