use cryptomkt::Client;

#[tokio::test]
async fn test_api_get_markets() {
    let api = Client::new("APK", "SK");
    let markets = api.get_markets();
    assert!(markets.await.len() > 1);
}
