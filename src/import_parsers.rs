use crate::constants;

mod heroic;
#[cfg(target_os = "linux")]
mod lutris;

pub fn handle_credentials_import(args: &crate::Args) -> (String, String, String) {
    if args.heroic {
        log::debug!("Loading Heroic credentials");
        let config = heroic::load_tokens();
        let config = config
            .fields
            .get(constants::GALAXY_CLIENT_ID)
            .expect("No Galaxy credentials");

        let access_token = config
            .get("access_token")
            .expect("access_token not present in heroic config")
            .as_str()
            .unwrap()
            .to_owned();
        let refresh_token = config
            .get("refresh_token")
            .expect("refresh_token not present in heroic config")
            .as_str()
            .unwrap()
            .to_owned();
        let galaxy_user_id = config
            .get("user_id")
            .expect("user_id not present in heroic config")
            .as_str()
            .unwrap()
            .to_owned();
        return (access_token, refresh_token, galaxy_user_id);
    }

    #[cfg(target_os = "linux")]
    if args.lutris {
        let config = lutris::load_tokens();
        let access_token = config
            .get("access_token")
            .expect("access_token not present in lutris config")
            .as_str()
            .unwrap()
            .to_owned();
        let refresh_token = config
            .get("refresh_token")
            .expect("refresh_token not present in lutris config")
            .as_str()
            .unwrap()
            .to_owned();
        let galaxy_user_id = config
            .get("user_id")
            .expect("user_id not present in lutris config")
            .as_str()
            .unwrap()
            .to_owned();
        return (access_token, refresh_token, galaxy_user_id);
    }

    let access_token = args.access_token.clone().expect("Access token is required");
    let refresh_token = args
        .refresh_token
        .clone()
        .expect("Refresh token is required");
    let galaxy_user_id = args.user_id.clone().expect("User id is required");

    (access_token, refresh_token, galaxy_user_id)
}
