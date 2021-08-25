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

    let migration_id = "gq8x3efLpEXTkGQchHTb";

    let result = ctn_client.asset_migration_outcome(
        migration_id,
    )?;

    match result.status {
        AssetMigrationStatus::Success => {
            // Asset amount successfully migrated
            println!("Asset amount successfully migrated");
        },
        AssetMigrationStatus::Pending => {
            // Final asset migration state not yet reached
        },
        _ => {
            // Asset migration has failed. Process error
            if let Some(error) = result.catenis_service.error {
                println!("Error executing Catenis service: {}", error);
            }

            if let Some(error) = result.foreign_transaction.error {
                println!("Error executing foreign blockchain transaction: {}", error);
            }
        },
    }

    Ok(())
}