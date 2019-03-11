use crate::db::Repo;
use crate::models::{Article, NewArticle};
use diesel::prelude::*;
use diesel::result::Error;
use std::str::FromStr;

use crate::schema::articles;
use crate::schema::users;

// joinable!(articles -> users (user_id));

#[derive(Default, Deserialize, Debug)]
pub struct ArticleQuery {
    pub author: Option<String>,
    pub favorited: Option<String>,
    pub tag: Option<String>,
}

impl FromStr for ArticleQuery {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_urlencoded::from_str::<ArticleQuery>(s).map_err(|e| e.to_string())
    }
}

pub async fn insert(repo: Repo, article: NewArticle) -> Result<Article, Error> {
    await! { repo.run(move |conn| {
        diesel::insert_into(articles::table)
            .values(&article)
            .get_result(&conn)
    })}
}

pub async fn find(repo: Repo, query: ArticleQuery) -> Result<Vec<Article>, Error> {
    use crate::schema::articles::dsl::*;
    use crate::schema::users::dsl::{username, users};

    await! { repo.run(move |conn| {
        let q = users.inner_join(articles)
        .select(articles::all_columns()).into_boxed();

        let q = if let Some(a) = query.author {
            q.filter(username.eq(a))
        } else {
            q
        };

        q.load::<Article>(&conn)
    } )}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{create_articles, create_users, init_env};
    use tokio_async_await_test::async_test;

    #[async_test]
    async fn test_list_articles() {
        init_env();
        let repo = Repo::new();

        let users = await! { create_users(&repo, 5) };
        let _articles = await! { create_articles(&repo, users)};
        let results =
            await! { find(repo.clone(), Default::default())}.expect("Failed to get articles");

        assert_eq!(results.len(), 5);
    }
}
