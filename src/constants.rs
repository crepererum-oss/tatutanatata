pub(crate) static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    ", revision ",
    env!("GIT_HASH")
);

pub(crate) static VERSION_STRING: &str =
    concat!(env!("CARGO_PKG_VERSION"), ", revision ", env!("GIT_HASH"));
