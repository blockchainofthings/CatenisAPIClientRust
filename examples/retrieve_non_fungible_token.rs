use catenis_api_client::{
    CatenisClient, ClientOptions, Environment, Result,
    api::*,
};

#[derive(Debug, Clone, Eq, PartialEq)]
struct NFTokenData {
    pub asset_id: Option<String>,
    pub metadata: Option<NonFungibleTokenMetadata>,
    pub contents: Option<Vec<String>>,
}

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

    let mut retrieve_options = RetrieveNonFungibleTokenOptions {
        retrieve_contents: None,
        contents_only: None,
        contents_encoding: None,
        data_chunk_size: Some(1024),
        async_: None,
        continuation_token: None,
    };
    let mut nf_token_data = NFTokenData {
        asset_id: None,
        metadata: None,
        contents: None,
     };
    let mut nf_token_contents = vec![];
    let token_id = "tDGQpGy627J6uAw4grYq";
    
    loop {
        let is_init_call = retrieve_options.continuation_token.is_none();
    
        let result = ctn_client.retrieve_non_fungible_token(token_id, retrieve_options.into())?;
    
        if let Some(nf_token) = result.non_fungible_token {
            if is_init_call {
                // Initial call. Get the token data
                nf_token_data.asset_id = nf_token.asset_id;
                nf_token_data.metadata = nf_token.metadata;
    
                if let Some(nft_contents) = nf_token.contents {
                    nf_token_contents.push(nft_contents.data);
                }
            } else if let Some(nft_contents) = nf_token.contents {
                // Add next contents part to token data
                nf_token_contents.push(nft_contents.data);
            }
        }
    
        if result.continuation_token.is_some() {
            // Whole contents data not yet retrieved. Prepare to get next part
            retrieve_options = RetrieveNonFungibleTokenOptions {
                retrieve_contents: None,
                contents_only: None,
                contents_encoding: None,
                data_chunk_size: None,
                async_: None,
                continuation_token: result.continuation_token,
            };
        } else {
            break;
        }
    }
    
    if nf_token_contents.len() > 0 {
        nf_token_data.contents = Some(nf_token_contents);
    }
    
    println!("Non-fungible token data: {:?}", nf_token_data);

    Ok(())
}