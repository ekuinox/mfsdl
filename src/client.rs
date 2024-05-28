use anyhow::{Context as _, Result};
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const BASE_URL: &str = "https://api.myfans.jp/api/v2";
const UA: &str = "user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36";

#[derive(Debug)]
pub struct MyfansClient {
    inner: reqwest::Client,
    headers: HeaderMap<HeaderValue>,
}

impl MyfansClient {
    pub fn new(token: String) -> Result<MyfansClient> {
        let mut headers = HeaderMap::with_capacity(3);
        headers.insert(
            header::AUTHORIZATION,
            format!("Token token={token}").parse()?,
        );
        headers.insert(header::USER_AGENT, UA.parse().unwrap());
        headers.insert("google-ga-data", "event328".parse().unwrap());
        Ok(MyfansClient {
            inner: reqwest::Client::new(),
            headers,
        })
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str, query: &impl Serialize) -> Result<T> {
        self.inner
            .get(format!("{BASE_URL}{path}"))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await
            .context("Failed to send GET request.")?
            .json::<T>()
            .await
            .context("Failed to deserialize response.")
            .map_err(From::from)
    }

    pub async fn post_ids_by_plan_id(
        &self,
        plan_id: &str,
        sort_key: &str,
        per_page: usize,
        page: usize,
    ) -> Result<(Vec<String>, Option<usize>)> {
        #[derive(Deserialize, Debug)]
        pub struct PostResponse {
            pub data: Vec<Post>,
            pub pagination: Pagination,
        }

        #[derive(Deserialize, Debug)]
        pub struct Post {
            pub id: String,
        }

        #[derive(Deserialize, Debug)]
        pub struct Pagination {
            pub current: usize,
            pub next: Option<usize>,
        }

        #[derive(Serialize, Debug)]
        pub struct Query {
            pub sort_key: String,
            pub per_page: usize,
            pub page: usize,
        }

        let PostResponse { data, pagination } = self
            .get(
                &format!("/plans/{plan_id}/posts"),
                &Query {
                    sort_key: sort_key.into(),
                    per_page,
                    page,
                },
            )
            .await?;

        let ids = data.into_iter().map(|post| post.id).collect::<Vec<_>>();
        Ok((ids, pagination.next))
    }
}