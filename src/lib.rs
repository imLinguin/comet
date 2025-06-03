#[macro_use]
extern crate lazy_static;

pub mod api;
pub mod config;
pub mod constants;
pub mod db;
pub mod paths;
pub mod proto;

pub static CERT: &[u8] = include_bytes!("../external/rootCA.pem");

lazy_static! {
    pub static ref CONFIG: config::Configuration = config::load_config().unwrap_or_default();
    pub static ref LOCALE: String = sys_locale::get_locale()
        .and_then(|x| if !x.contains("-") { None } else { Some(x) })
        .unwrap_or_else(|| String::from("en-US"));
}

