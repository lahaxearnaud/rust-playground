use reqwest::{Client, Error};

pub fn build_client() -> Result<Client, Error> {
    let client_promise = reqwest::Client::builder()
    //    .proxy(reqwest::Proxy::https("http://pnpxyu.boursorama.fr:3128").ok().unwrap())
        .build();

    return client_promise;
}
