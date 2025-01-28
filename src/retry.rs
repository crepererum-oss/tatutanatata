use std::{future::Future, time::Duration};

use anyhow::{Context, Result};
use rand::{rng, rngs::StdRng, Rng, RngCore, SeedableRng};
use tracing::warn;

/// Exponential backoff with jitter
///
/// See <https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/>
pub(crate) async fn retry<F, Fut, R, T, E>(what: &'static str, f: F, should_retry: R) -> Result<T>
where
    F: Fn() -> Fut + Send,
    Fut: Future<Output = Result<T, E>> + Send,
    R: for<'a> Fn(&'a E) -> bool + Send,
    T: Send,
    E: std::error::Error + Send + Sync + 'static,
{
    let config = Config::default();
    tokio::time::timeout(config.deadline, async move {
        for sleep in Sleep::from(config) {
            match f().await {
                Ok(x) => {
                    return Ok(x);
                }
                Err(e) if should_retry(&e) => {
                    warn!(%e, what, sleep_sec=sleep.as_secs_f64(), "retry");
                    tokio::time::sleep(sleep).await;
                }
                Err(e) => {
                    return Err(e).context("failed");
                }
            }
        }

        unreachable!("iterator never ends")
    })
    .await
    .context("deadline exceeded")?
}

struct Config {
    multiplier: f64,
    cap: Duration,
    base: Duration,
    deadline: Duration,
    rng: Box<dyn RngCore + Send + Sync>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            multiplier: 3.0,
            cap: Duration::from_secs(60),
            base: Duration::from_millis(100),
            deadline: Duration::from_secs(600),
            rng: Box::new(StdRng::from_rng(&mut rng())),
        }
    }
}

struct Sleep {
    config: Config,
    sleep: Duration,
}

impl From<Config> for Sleep {
    fn from(config: Config) -> Self {
        let sleep = config.base;
        Self { config, sleep }
    }
}

impl Iterator for Sleep {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        let sleep = self.sleep;
        self.sleep = self
            .config
            .cap
            .min(Duration::from_secs_f64(self.config.rng.random_range(
                self.config.base.as_secs_f64()..(sleep.as_secs_f64() * self.config.multiplier),
            )));
        Some(sleep)
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::mock::StepRng;

    use super::*;

    #[test]
    fn test_sleep() {
        let mut it = Sleep::from(Config {
            rng: Box::new(StepRng::new(15397336220812216376, u64::MAX / 2 - 1)),
            cap: Duration::from_secs(2),
            ..Default::default()
        });

        assert_approx_eq(it.next().unwrap().as_secs_f64(), 0.1);
        assert_approx_eq(it.next().unwrap().as_secs_f64(), 0.266938254);
        assert_approx_eq(it.next().unwrap().as_secs_f64(), 0.334556582);
        assert_approx_eq(it.next().unwrap().as_secs_f64(), 0.854285247);
        assert_approx_eq(it.next().unwrap().as_secs_f64(), 0.924296313);
        assert_approx_eq(it.next().unwrap().as_secs_f64(), 2.0);
        assert_approx_eq(it.next().unwrap().as_secs_f64(), 2.0);
    }

    #[track_caller]
    fn assert_approx_eq(a: f64, b: f64) {
        assert!((a - b).abs() < 0.000000001, "{a} != {b}",);
    }
}
