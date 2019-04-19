use std::{collections::HashMap, fs};

use chrono::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use crate::prelude::*;
use crate::schema::groups;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub groups: HashMap<String, String>,
}

#[derive(Debug, Insertable, Queryable)]
#[table_name = "groups"]
struct Group {
    id: Uuid,
    display_name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub fn run(database_url: &str, config_path: &str) -> Result<()> {
    let content = fs::read(config_path)?;
    let config = toml::from_slice::<Config>(&content)?;
    let conn = PgConnection::establish(database_url)?;

    init_groups(&conn, config.groups)?;

    Ok(())
}

fn init_groups(
    conn: &PgConnection,
    groups: HashMap<String, String>,
) -> Result<()> {
    for (name, desc) in groups {
        let group = get_or_create_group(&conn, &name)?;

        match group.description {
            Some(ref description) if description == &desc => continue,
            _ => update_group_desc(&conn, &group.id, &desc)?,
        }
    }

    Ok(())
}

fn get_or_create_group(conn: &PgConnection, name: &str) -> Result<Group> {
    use crate::schema::groups::dsl::*;

    match groups
        .filter(display_name.eq(name))
        .first::<Group>(conn)
        .optional()?
    {
        Some(group) => Ok(group),
        None => {
            let now = Utc::now();
            let result = diesel::insert_into(groups)
                .values(&Group {
                    id: Uuid::new_v4(),
                    display_name: name.to_string(),
                    description: None,
                    created_at: now,
                    updated_at: now,
                })
                .get_result::<Group>(conn)?;
            Ok(result)
        }
    }
}

fn update_group_desc(
    conn: &PgConnection,
    group_id: &Uuid,
    desc: &str,
) -> Result<()> {
    use crate::schema::groups::dsl::*;

    let _ = diesel::update(groups.find(group_id))
        .set((description.eq(desc), updated_at.eq(Utc::now())))
        .execute(conn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    impl Config {
        pub fn new() -> Self {
            Config {
                groups: HashMap::new(),
            }
        }

        pub fn insert_group(&mut self, key: String, val: String) -> &mut Self {
            self.groups.insert(key, val);
            self
        }
    }

    fn input_config() -> Config {
        let config: Config = toml::from_str(r#"
                                            [groups]
                                            user  = "Act as a user in the system"
                                            admin = "Act as an administrator throughout the system"
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
        use crate::schema::groups::dsl::*;

        let conn = connection();
        let input_config = input_config();
        let mut expected_groups = expected_config().groups;

        init_groups(&conn, input_config.groups).unwrap();

        for group in groups.load::<Group>(&conn).unwrap() {
            let expected_desc = expected_groups.remove(&group.display_name);

            assert_eq!(&expected_desc, &group.description);
        }
    }
}
