table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        password -> Text,
        avatar_url -> Text,
        nickname -> Text,
        is_verified -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
