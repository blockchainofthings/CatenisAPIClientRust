use catenis_api_client::{
    CatenisClient, ClientOptions, Environment, Result,
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

    let result = ctn_client.list_owned_assets(
        Some(200),
        Some(0),
    )?;

    for idx in 0..result.owned_assets.len() {
        let owned_asset = &result.owned_assets[idx];

        println!("Owned asset #{}:", idx + 1);
        println!(" - asset ID: {}", owned_asset.asset_id);
        println!(" - current asset balance: {}", owned_asset.balance.total);
        println!(" - amount not yet confirmed: {}", owned_asset.balance.unconfirmed);
    }

    if result.has_more {
        println!("Not all owned assets have been returned");
    }

    Ok(())
}