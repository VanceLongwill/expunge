use expunge::Expunge;

fn redact_first_char(mut s: String) -> String {
    s.replace_range(0..1, "*");
    s
}

fn char_count_of(s: String) -> String{
    s.len().to_string()
}

#[derive(Expunge)]
#[cfg_attr(test, derive(Eq, PartialEq, Debug), expunge(allow_debug))]
struct User {
  username: String,
  #[expunge(with = char_count_of)]
  first_name: String,
  #[expunge(with = redact_first_char)]
  last_name: String,
  #[expunge(with = sha256::digest)]
  password: String,
}

#[test]
fn field_with() {
    let user = User {
        username: "some_user_123".to_string(),
        first_name: "Jane".to_string(),
        last_name: "Doe".to_string(),
        password: "password123".to_string(),
    };

    assert_eq!(User{
        username: "".to_string(),
        first_name: "4".to_string(),
        last_name: "*oe".to_string(),
        password: "ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f".to_string(),
    }, user.expunge());
}
