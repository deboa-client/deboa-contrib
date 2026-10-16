use caramelo::{
    expect,
    matchers::{eq, err},
};
use deboa::TestResult;
use deboa_extras::hook::{
    redirect::Redirect,
    retry::{Retry, Sleeper},
};
use deboa_tokio::{Client, request::get};
use http::Version;

#[tokio::test]
async fn test_redirect() -> TestResult<()> {
    let client = Client::default().chain(Redirect::default());

    let response = get("https://uol.com")?
        .version(Version::HTTP_11)
        .send_with(&client)
        .await?;

    expect(response.status()).to_be(eq(200));

    Ok(())
}

#[derive(Clone, Copy, Default)]
struct TokioSleeper;

impl Sleeper for TokioSleeper {
    async fn sleep(&self, duration: std::time::Duration) {
        tokio::time::sleep(duration).await;
    }
}

#[tokio::test]
async fn test_retry() -> TestResult<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let client = Client::default().chain(Retry::<TokioSleeper>::default());

    let response = get("https://api.uptimesignal.io/test/500")?
        .version(Version::HTTP_11)
        .send_with(&client)
        .await;

    expect(response).to_be(err());

    Ok(())
}
