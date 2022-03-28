use tokio;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpwardError {
    #[error("the API for key `{0}` is not available")]
    MissingKey(String),
    #[error("unknown data store error")]
    Unknown,
}
#[tokio::main]
async fn main() {

    let influx_url =    dotenvy::var("SERVER_URL").unwrap();
    let _org = dotenvy::var("ORG").unwrap();
    let org_id = dotenvy::var("ORGID").unwrap();
    let token = dotenvy::var("INFLUX_AUTH_TOKEN").unwrap();
    let bucket = dotenvy::var("BUCKET").unwrap();
    let query = format!("from(bucket: \"{bucket}\") |> range(start: -1h) |> filter(fn: (r) => r.isin ==\"CA94947L1022\")");

    let req_url = format!(r"{}/api/v2/query\?orgID={}", influx_url, &org_id);
    let client = reqwest::Client::new();
    let response = client.post( &req_url)
    .header("Authorization", format!("Token {}", token))
    .header("Accept", "application/csv")
    .header("Content-Type", "application/vnd.flux")
    .body(query)
    .send()
    .await.unwrap().text().await.unwrap();


        println!("response: {response}" );

}
