use expunge::Expunge;

#[derive(Default)]
struct Location(f64, f64);

#[derive(Expunge)]
#[expunge(default)]
struct UserData {
  username: String,
  password: String,
  location: Location,
}
