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

    let asset_id = "ahfTzqgWAXnMR6Z57mcp";

    let result = ctn_client.reissue_non_fungible_asset(
        asset_id,
        NFAssetReissuanceInfoOrContToken::ReissuanceInfo(NonFungibleAssetReissuanceInfo {
            encrypt_nft_contents: None,
            holding_devices: None,
            async_: None,
        }),
        Some(vec![
            NewNonFungibleTokenInfo {
                metadata: Some(NewNonFungibleTokenMetadata {
                    name: String::from("NFA1 NFT 3"),
                    description: Some(String::from("Third token of non-fungible asset #1")),
                    custom: None,
                }),
                contents: Some(NewNonFungibleTokenContents {
                    data: String::from("Contents of third token of non-fungible asset #1"),
                    encoding: Encoding::UTF8
                }),
            },
            NewNonFungibleTokenInfo {
                metadata: Some(NewNonFungibleTokenMetadata {
                    name: String::from("NFA1 NFT 4"),
                    description: Some(String::from("Forth token of non-fungible asset #1")),
                    custom: None,
                }),
                contents: Some(NewNonFungibleTokenContents {
                    data: String::from("Contents of forth token of non-fungible asset #1"),
                    encoding: Encoding::UTF8
                }),
            },
        ]),
        Some(true)
    )?;

    println!("IDs of newly issued non-fungible tokens: {:?}", result.nf_token_ids.unwrap());

    Ok(())
}