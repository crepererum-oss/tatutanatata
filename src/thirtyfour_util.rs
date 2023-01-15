use anyhow::{anyhow, ensure, Context, Result};
use async_trait::async_trait;
use thirtyfour::{session::handle::SessionHandle, By, WebElement};

#[async_trait]
pub trait FindExt {
    async fn find_all_ext(&self, by: impl Into<By> + Send) -> Result<Vec<WebElement>>;

    async fn find_one(&self, by: impl Into<By> + Send) -> Result<WebElement> {
        let mut results = self.find_all_ext(by).await?;
        ensure!(
            results.len() == 1,
            "expected exactly one element but got {}",
            results.len()
        );
        Ok(results.remove(0))
    }

    async fn find_at_most_one(&self, by: impl Into<By> + Send) -> Result<Option<WebElement>> {
        let mut results = self.find_all_ext(by).await?;
        ensure!(
            results.len() <= 1,
            "expected at most one element but got {}",
            results.len()
        );
        if results.is_empty() {
            Ok(None)
        } else {
            Ok(Some(results.remove(0)))
        }
    }

    async fn find_one_with_attr(
        &self,
        by: impl Into<By> + Send,
        attr_name: &str,
        attr_value: &str,
    ) -> Result<WebElement> {
        let elements = self.find_all_ext(by).await.context("find elements")?;
        let mut found = None;
        for element in elements {
            if let Some(v) = element
                .attr(attr_name)
                .await
                .context("element attr")?
                .as_deref()
            {
                if v == attr_value {
                    ensure!(found.is_none(), "multiple matching elements");
                    found = Some(element);
                }
            }
        }
        let found = found.ok_or_else(|| anyhow!("not found"))?;
        Ok(found)
    }
}

#[async_trait]
impl FindExt for SessionHandle {
    async fn find_all_ext(&self, by: impl Into<By> + Send) -> Result<Vec<WebElement>> {
        let results = self.find_all(by).await.context("find all")?;

        Ok(results)
    }
}

#[async_trait]
impl FindExt for WebElement {
    async fn find_all_ext(&self, by: impl Into<By> + Send) -> Result<Vec<WebElement>> {
        let results = self.find_all(by).await.context("find all")?;

        Ok(results)
    }
}
