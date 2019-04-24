use std::{collections::HashMap, fs};

use diesel::prelude::*;

use crate::db::{groups, users};
use crate::error::BootstrapError as Error;
use crate::types::GroupMembershipType;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub groups: HashMap<String, String>,
    pub users: HashMap<String, String>,
}

pub fn run(database_url: &str, config_path: &str) -> Result<(), Error> {
    let content = fs::read(config_path)?;
    let config = toml::from_slice::<Config>(&content)?;
    let conn = PgConnection::establish(database_url)?;

    init_groups(&conn, config.groups)?;
    init_users(&conn, config.users)?;

    Ok(())
}

fn init_groups(
    conn: &PgConnection,
    groups: HashMap<String, String>,
) -> Result<(), Error> {
    for (name, desc) in groups {
        let group = groups::get_or_create(&conn, &name)?;

        match group.description {
            Some(ref description) if description == &desc => continue,
            _ => groups::update_desc(&conn, &group.id, &desc)?,
        }
    }

    Ok(())
}

fn init_users(
    conn: &PgConnection,
    users: HashMap<String, String>,
) -> Result<(), Error> {
    for (ref username, ref user_info) in users {
        let (nickname, password, groups) = parse_user_info(user_info)?;
        let user = users::create_or_update(conn, username, nickname, password)?;

        for g_name in groups {
            let group = groups::get_or_create(conn, g_name)?;
            let _ = groups::add_member(
                conn,
                &group.id,
                &user.id,
                GroupMembershipType::User,
            )?;
        }
    }

    Ok(())
}

fn parse_user_info(user_info: &str) -> Result<(&str, &str, Vec<&str>), Error> {
    let info = user_info.split('|').collect::<Vec<&str>>();

    let result = match info.len() {
        2 => (info[0], info[1], Vec::new()),
        3 => (info[0], info[1], info[2].split(',').collect()),
        _ => return Err(Error::UserInfoParse(user_info.to_string())),
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
        use crate::schema::groups;
        use crate::types::Group;

        let conn = connection();
        let input_config = input_config();
        let mut expected_groups = expected_config().groups;

        init_groups(&conn, input_config.groups).unwrap();

        for group in groups::table.load::<Group>(&conn).unwrap() {
            let expected_desc = expected_groups.remove(&group.display_name);

            assert_eq!(&expected_desc, &group.description);
        }
    }

    #[test]
    fn test_init_users() {
        use crate::schema::{group_membership, groups, users};
        use crate::types::User;

        let conn = connection();
        let input_config = input_config();
        let mut expected_users = expected_config().users;

        init_users(&conn, input_config.users).unwrap();

        for user in users::table.load::<User>(&conn).unwrap() {
            let user_info = expected_users.remove(&user.username).unwrap();
            let (expected_nickname, raw_password, expected_groups) =
                parse_user_info(&user_info).unwrap();

            let password_verified =
                utils::verify_password(&raw_password, &user.password).unwrap();

            let groups = group_membership::table
                .inner_join(groups::table)
                .select(groups::display_name)
                .filter(group_membership::member_id.eq(&user.id))
                .load::<String>(&conn)
                .unwrap();

            let groups =
                groups.iter().map(|s| s.as_ref()).collect::<Vec<&str>>();

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
