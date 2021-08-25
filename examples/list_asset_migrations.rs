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

    let result = ctn_client.list_asset_migrations(
        Some(ListAssetMigrationsOptions {
            asset_id: None,
            foreign_blockchain: Some(ForeignBlockchain::Ethereum),
            direction: Some(AssetMigrationDirection::Outward),
            status: Some(vec![AssetMigrationStatus::Success]),
            negate_status: None,
            start_date: Some("2021-08-01T00:00:00Z".into()),
            end_date: None,
            limit: Some(200),
            skip: Some(0),
        }),
    )?;

    if result.asset_migrations.len() > 0 {
        println!("Returned asset migrations: {:?}", result.asset_migrations);

        if result.has_more {
            println!("Not all asset migrations have been returned");
        }
    }

    Ok(())
}