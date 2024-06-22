use expunge::Expunge;

#[derive(Expunge)]
struct UserLogin {
    username: String,
    password: String,
    #[expunge(skip)]
    last_logged_in_at: i64, // the last login timestamp will be left as-is
}

#[test]
fn skip() {
    let login = UserLogin {
        username: "gamer100".to_string(),
        password: "somepassword123".to_string(),
        last_logged_in_at: 1716113380,
    };

    let expunged = login.expunge();
    assert_eq!("", expunged.username);
    assert_eq!("", expunged.password);
    assert_eq!(1716113380, expunged.last_logged_in_at);
}
