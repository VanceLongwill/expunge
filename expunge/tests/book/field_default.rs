use expunge::Expunge;

#[derive(Default)]
struct Location(f64, f64);

#[derive(Expunge)]
struct UserData {
  username: String,
  password: String,
  #[expunge(default)]
  location: Location,
}
