use std::{collections::HashMap, fs};

use diesel::{Connection, PgConnection};

use crate::error::{ErrorKind, Result, ResultExt};
use crate::groups::{self, GroupMembershipType};
use crate::users;
use crate::utils;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub groups: HashMap<String, String>,
    pub users: HashMap<String, String>,
}

pub fn run(database_url: &str, config_path: &str) -> Result<()> {
    let content = fs::read(config_path).context(ErrorKind::BootstrapError)?;
    let config = toml::from_slice::<Config>(&content)
        .context(ErrorKind::BootstrapError)?;
    let conn = PgConnection::establish(database_url)
        .context(ErrorKind::BootstrapError)?;

    init_groups(&conn, config.groups)?;
    init_users(&conn, config.users)?;

    Ok(())
}

fn init_groups(
    conn: &PgConnection,
    groups: HashMap<String, String>,
) -> Result<()> {
    for (name, desc) in groups {
        let group = groups::get_or_create(&conn, &name)
            .context(ErrorKind::BootstrapError)?;

        match group.description {
            Some(ref description) if description == &desc => continue,
            _ => groups::update_desc(&conn, &group.id, &desc)
                .context(ErrorKind::BootstrapError)?,
        };
    }

    Ok(())
}

fn init_users(
    conn: &PgConnection,
    users: HashMap<String, String>,
) -> Result<()> {
    for (ref username, ref user_info) in users {
        let (nickname, password, groups) = parse_user_info(user_info)?;
        let hashed_password = utils::hash_password(password)
            .context(ErrorKind::BootstrapError)?;
        let user =
            users::create_or_update(conn, username, nickname, &hashed_password)
                .context(ErrorKind::BootstrapError)?;
        groups::del_members_by_member_id(conn, &user.id)
            .context(ErrorKind::BootstrapError)?;

        for g_name in groups {
            let group = groups::get_or_create(conn, g_name)
                .context(ErrorKind::BootstrapError)?;
            let _ = groups::add_member(
                conn,
                &group.id,
                &user.id,
                GroupMembershipType::User,
            )
            .context(ErrorKind::BootstrapError)?;
        }
    }

    Ok(())
}

fn parse_user_info(user_info: &str) -> Result<(&str, &str, Vec<&str>)> {
    let info = user_info.split('|').collect::<Vec<&str>>();

    let result = match info.len() {
        2 => (info[0], info[1], Vec::new()),
        3 => (info[0], info[1], info[2].split(',').collect()),
        _ => Err(ErrorKind::BootstrapError)?,
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use crate::utils;

    impl Config {
        pub fn new() -> Self {
            Config {
                groups: HashMap::new(),
                users: HashMap::new(),
            }
        }

        pub fn insert_group(&mut self, key: String, val: String) -> &mut Self {
            self.groups.insert(key, val);
            self
        }

        pub fn insert_user(&mut self, key: String, val: String) -> &mut Self {
            self.users.insert(key, val);
            self
        }
    }

    fn input_config() -> Config {
        let config: Config = toml::from_str(r#"
                                            [groups]
                                            user  = "Act as a user in the system"
                                            admin = "Act as an administrator throughout the system"

                                            [users]
                                            bob = "Bob|123456|admin,user"
                                            "#).unwrap();

        config
    }

    fn expected_config() -> Config {
        let mut config = Config::new();
        config.insert_group(
            "user".to_string(),
            "Act as a user in the system".to_string(),
        );
        config.insert_group(
            "admin".to_string(),
            "Act as an administrator throughout the system".to_string(),
        );

        config.insert_user(
            "bob".to_string(),
            "Bob|123456|admin,user".to_string(),
        );

        config
    }

    #[test]
    fn test_parse_config() {
        let input_config = input_config();
        let expected_config = expected_config();
        assert_eq!(&expected_config, &input_config);
    }

    #[test]
    fn test_init_groups() {
        let conn = connection();
        let input_config = input_config();
        let expected_groups = expected_config().groups;

        init_groups(&conn, input_config.groups).unwrap();

        for (display_name, description) in expected_groups {
            let group =
                groups::find_by_name(&conn, &display_name).unwrap().unwrap();

            assert_eq!(Some(description), group.description);
        }
    }

    #[test]
    fn test_init_users() {
        let conn = connection();
        let input_config = input_config();
        let expected_users = expected_config().users;

        init_users(&conn, input_config.users).unwrap();

        for (username, user_info) in expected_users {
            let user =
                users::find_by_username(&conn, &username).unwrap().unwrap();
            let (expected_nickname, raw_password, expected_groups) =
                parse_user_info(&user_info).unwrap();

            let password_verified =
                utils::verify_password(&raw_password, &user.password).unwrap();

            let groups = groups::find_by_member_id(&conn, &user.id)
                .unwrap()
                .into_iter()
                .map(|g| g.display_name)
                .collect::<Vec<String>>();

            assert!(password_verified);
            assert_eq!(&expected_nickname, &user.nickname);
            assert_eq!(&expected_groups, &groups);
        }
    }

    #[test]
    fn test_parse_user_info() {
        let (nickname, password, groups) =
            parse_user_info("Bob|123456|admin,user").unwrap();

        assert_eq!(&"Bob", &nickname);
        assert_eq!(&"123456", &password);
        assert_eq!(&vec!["admin", "user"], &groups);
    }
}
