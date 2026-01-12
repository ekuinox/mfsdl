use anyhow::{Context as _, Result};
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use structstruck::strike;

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

    pub async fn get<T: DeserializeOwned>(
        &self,
        path: impl AsRef<str>,
        query: &impl Serialize,
    ) -> Result<T> {
        let path = path.as_ref();
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
    }

    pub async fn get_post_ids_by_plan_id(
        &self,
        plan_id: &str,
        sort_key: &str,
        per_page: usize,
        page: usize,
    ) -> Result<(Vec<String>, Option<usize>)> {
        strike! {
            #[strikethrough[derive(Deserialize, Debug)]]
            struct PostResponse {
                data: Vec<struct Post {
                   id: String,
                }>,
                pagination: struct Pagination {
                    next: Option<usize>,
                },
            }
        }

        #[derive(Serialize, Debug)]
        struct Query {
            sort_key: String,
            per_page: usize,
            page: usize,
        }

        let PostResponse { data, pagination } = self
            .get(
                format!("/plans/{plan_id}/posts"),
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

    pub async fn get_post_video_url(&self, post_id: &str) -> Result<Option<String>> {
        strike! {
            #[strikethrough[derive(Deserialize, Debug)]]
            struct VideoResponse {
                main: Option<Vec<struct Video {
                    // .m3u8
                    url: String,
                    width: usize,
                }>>,
            }
        }

        let VideoResponse { main } = self.get(format!("/posts/{post_id}/videos"), &()).await?;

        let mut main = main.unwrap_or_default();
        main.sort_by(|a, b| a.width.cmp(&b.width));
        main.reverse();

        Ok(main.into_iter().next().map(|video| video.url))
    }
}
