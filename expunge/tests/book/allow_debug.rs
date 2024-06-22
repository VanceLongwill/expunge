use expunge::Expunge;

#[derive(Expunge)]
#[cfg_attr(test, derive(Debug), expunge(allow_debug))]
struct Credentials {
    username: String,
    private_key: String,
}
