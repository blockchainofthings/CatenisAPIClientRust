use catenis_api_client::{
    CatenisClient, ClientOptions, Environment, Result,
    api::*,
};

fn main() -> Result<()> {
    let device_credentials = (
        "dnN3Ea43bhMTHtTvpytS",
        concat!(
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
        ),
    ).into();

    let mut ctn_client = CatenisClient::new_with_options(
        Some(device_credentials),
        &[
            ClientOptions::Environment(Environment::Sandbox),
        ],
    )?;

    let asset_id = "aH2AkrrL55GcThhPNa3J";

    let result = ctn_client.migrate_asset(
        asset_id,
        ForeignBlockchain::Ethereum,
        AssetMigration::Info(AssetMigrationInfo {
            direction: AssetMigrationDirection::Outward,
            amount: 50.5,
            dest_address: Some(String::from("0xe247c9BfDb17e7D8Ae60a744843ffAd19C784943")),
        }),
        None,
    )?;

    println!("Pending asset migration: {:?}", result);

    // Start polling for asset migration outcome

    Ok(())
}