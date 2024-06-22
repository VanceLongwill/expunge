use expunge::Expunge;

#[derive(Expunge)]
#[expunge(with = sha256::digest)]
struct Credential {
  username: String,
  private_key: String,
}
