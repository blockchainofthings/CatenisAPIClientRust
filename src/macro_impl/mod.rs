#[cfg(test)]
macro_rules! now {
    () => { test_helper::time::now() }
}
#[cfg(not(test))]
macro_rules! now {
    () => { time::OffsetDateTime::now_utc() }
}

/// Helper for defining custom non-fungible token metadata properties.
///
/// Pass a JSON literal as the single input parameter, and the macro will return the corresponding
/// `Option<`[`JsonObject`](crate::JsonObject)`>` value.
///
/// # Example
/// ```no_run
/// use catenis_api_client::{
///     json_obj,
///     api::{
///         IntoJsonObj,
///         NewNonFungibleTokenMetadata,
///     }
/// };
///
/// let new_nf_token_metadata = NewNonFungibleTokenMetadata {
///     name: String::from("TestNFToken_1"),
///     description: Some(String::from("First non-fungible token issued for test")),
///     custom: json_obj!({
///         "sensitiveProps": {
///             "senseProp1": "XYZ",
///             "senseProp2": "456"
///         },
///         "propNum": 5,
///         "propStr": "ABC",
///         "propBool": true
///     }),
/// };
/// ```
#[macro_export]
macro_rules! json_obj {
    ($($json:tt)+) => {
        serde_json::json!($($json)+).json_obj()
    };
}
