pub const APP_NAME: &str = "STU";

#[cfg(not(test))]
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(not(test))]
pub const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
#[cfg(not(test))]
pub const APP_REPOSITORY_URL: &str = env!("CARGO_PKG_REPOSITORY");

#[cfg(test)]
pub const APP_VERSION: &str = "1.2.3";
#[cfg(test)]
pub const APP_DESCRIPTION: &str = "S3 Terminal UI";
#[cfg(test)]
pub const APP_REPOSITORY_URL: &str = "http://example.com/stu";
