// @generated automatically by Diesel CLI.

diesel::table! {
    quotes (id) {
        id -> Text,
        author -> Text,
        quote -> Text,
    }
}
