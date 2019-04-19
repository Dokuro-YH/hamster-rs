table! {
    group_membership (group_id, member_id) {
        group_id -> Uuid,
        member_id -> Uuid,
        member_type -> Text,
        added -> Timestamptz,
    }
}

table! {
    groups (id) {
        id -> Uuid,
        display_name -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

joinable!(group_membership -> groups (group_id));

allow_tables_to_appear_in_same_query!(
    group_membership,
    groups,
);
