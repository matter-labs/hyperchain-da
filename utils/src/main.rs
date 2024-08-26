// Make a bin crate that calls the near client, sends a request to the near client, and prints the response.
// Also fetches the proof from the near client and prints the proof.

use dotenvy::dotenv;
use hyperchain_da::clients::near::client::NearClient;
use zksync_da_client::DataAvailabilityClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let near_client = NearClient::new().await?;

    // let dispatch_response = near_client
    //     .dispatch_blob(1, vec![1, 2, 3, 4])
    //     .await
    //     .unwrap();

    // println!("Transaction ID: {:?}", dispatch_response.blob_id);

    let proof = near_client
        .get_inclusion_data("D9urW88xYRGnEHQD1u2zUsPatuieSPBnzSPj82hvGt2L")
        .await
        .unwrap();

    if let Some(data) = proof {
        println!("Proof: {:?}", data.data);
    } else {
        println!("No proof found");
    }

    Ok(())
}
