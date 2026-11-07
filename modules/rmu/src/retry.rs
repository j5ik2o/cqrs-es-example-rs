use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use rand::Rng;

/// Sleeps for a delay. Tests use a fake so retry tests do not spend real time.
#[async_trait]
pub trait Sleeper: Send + Sync {
  async fn sleep(&self, delay: Duration);
}

pub struct SystemSleeper;

#[async_trait]
impl Sleeper for SystemSleeper {
  async fn sleep(&self, delay: Duration) {
    tokio::time::sleep(delay).await;
  }
}

pub type Jitter = Arc<dyn Fn(Duration) -> Duration + Send + Sync>;

/// Full jitter returns a random delay between zero and the given delay.
pub fn full_jitter(delay: Duration) -> Duration {
  if delay.is_zero() {
    return Duration::ZERO;
  }
  let upper = delay.as_millis().saturating_add(1);
  let millis = rand::thread_rng().gen_range(0..upper);
  Duration::from_millis(millis as u64)
}

/// Bounds how long one record may be retried before it is parked.
///
/// `max_elapsed` is a wall-clock budget, so a long queue of records cannot push
/// one invocation past the caller's deadline.
#[derive(Clone)]
pub struct RetryPolicy {
  max_attempts: usize,
  max_elapsed: Duration,
  initial_delay: Duration,
  max_delay: Duration,
  jitter: Jitter,
  sleeper: Arc<dyn Sleeper>,
}

impl std::fmt::Debug for RetryPolicy {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RetryPolicy")
      .field("max_attempts", &self.max_attempts)
      .field("max_elapsed", &self.max_elapsed)
      .field("initial_delay", &self.initial_delay)
      .field("max_delay", &self.max_delay)
      .finish_non_exhaustive()
  }
}

impl RetryPolicy {
  pub fn new(max_attempts: usize, max_elapsed: Duration, initial_delay: Duration, max_delay: Duration) -> Self {
    Self {
      max_attempts: max_attempts.max(1),
      max_elapsed,
      initial_delay,
      max_delay,
      jitter: Arc::new(full_jitter),
      sleeper: Arc::new(SystemSleeper),
    }
  }

  /// Default bounds for the Lambda updater: five attempts within thirty
  /// seconds, delays from 200ms capped at 5s with full jitter.
  pub fn lambda_default() -> Self {
    Self::new(
      5,
      Duration::from_secs(30),
      Duration::from_millis(200),
      Duration::from_secs(5),
    )
  }

  /// Default bounds for the local reader: five attempts within thirty seconds,
  /// delays from 500ms capped at 30s with full jitter.
  pub fn local_default() -> Self {
    Self::new(
      5,
      Duration::from_secs(30),
      Duration::from_millis(500),
      Duration::from_secs(30),
    )
  }

  /// Used when the outer delivery mechanism owns retries.
  pub fn no_retry() -> Self {
    Self::new(1, Duration::ZERO, Duration::ZERO, Duration::ZERO)
  }

  pub fn with_sleeper(mut self, sleeper: Arc<dyn Sleeper>) -> Self {
    self.sleeper = sleeper;
    self
  }

  pub fn with_jitter(mut self, jitter: Jitter) -> Self {
    self.jitter = jitter;
    self
  }

  pub fn max_attempts(&self) -> usize {
    self.max_attempts
  }

  /// The jittered delay before the next attempt. `attempt` starts at 1.
  pub fn delay(&self, attempt: usize) -> Duration {
    let attempt = attempt.max(1);
    if self.initial_delay.is_zero() {
      return Duration::ZERO;
    }
    let mut delay = self.initial_delay;
    for _ in 1..attempt {
      if !self.max_delay.is_zero() && delay >= self.max_delay {
        break;
      }
      delay = delay.saturating_mul(2);
    }
    if !self.max_delay.is_zero() && delay > self.max_delay {
      delay = self.max_delay;
    }
    (self.jitter)(delay)
  }

  /// Runs `op` until it succeeds, fails permanently, or the policy is exhausted.
  pub async fn run<T, E, F, Fut, P>(&self, mut op: F, is_retryable: P) -> Result<T, RetryError<E>>
  where
    E: std::fmt::Debug,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: Fn(&E) -> bool, {
    let attempts = self.max_attempts.max(1);
    let started_at = Instant::now();
    let mut last_error = None;
    for attempt in 1..=attempts {
      match op().await {
        Ok(value) => return Ok(value),
        Err(error) => {
          let retryable = is_retryable(&error);
          if !retryable || attempt == attempts {
            return Err(RetryError {
              attempts: attempt,
              elapsed: started_at.elapsed(),
              permanent: !retryable,
              source: error,
            });
          }
          let delay = self.delay(attempt);
          if !self.max_elapsed.is_zero() && started_at.elapsed().saturating_add(delay) > self.max_elapsed {
            return Err(RetryError {
              attempts: attempt,
              elapsed: started_at.elapsed(),
              permanent: false,
              source: error,
            });
          }
          self.sleeper.sleep(delay).await;
          last_error = Some(error);
        }
      }
    }
    Err(RetryError {
      attempts,
      elapsed: started_at.elapsed(),
      permanent: false,
      source: last_error.expect("at least one attempt ran"),
    })
  }
}

#[derive(Debug)]
pub struct RetryError<E: std::fmt::Debug> {
  pub attempts: usize,
  pub elapsed: Duration,
  pub permanent: bool,
  pub source: E,
}

impl<E: std::fmt::Display + std::fmt::Debug> std::fmt::Display for RetryError<E> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "retry exhausted after {} attempts in {:?}: {}",
      self.attempts, self.elapsed, self.source
    )
  }
}

impl<E: std::error::Error + 'static> std::error::Error for RetryError<E> {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(&self.source)
  }
}

/// The reader's idle and failure delay. It grows exponentially up to a cap and
/// applies full jitter, so a restarted reader does not hammer the stream or the
/// database in lock step with another replica.
pub struct Backoff {
  initial: Duration,
  max: Duration,
  attempt: u32,
  jitter: Jitter,
}

impl Backoff {
  pub fn new(initial: Duration, max: Duration) -> Self {
    Self::with_jitter(initial, max, Arc::new(full_jitter))
  }

  pub fn with_jitter(initial: Duration, max: Duration, jitter: Jitter) -> Self {
    let initial = if initial.is_zero() {
      Duration::from_millis(500)
    } else {
      initial
    };
    let max = if max.is_zero() || max < initial {
      Duration::from_secs(30)
    } else {
      max
    };
    Self {
      initial,
      max,
      attempt: 0,
      jitter,
    }
  }

  pub fn next_delay(&mut self) -> Duration {
    let mut delay = self.initial;
    for _ in 0..self.attempt {
      if delay >= self.max {
        break;
      }
      delay = delay.saturating_mul(2);
    }
    if delay > self.max {
      delay = self.max;
    }
    self.attempt = self.attempt.saturating_add(1);
    (self.jitter)(delay)
  }

  pub fn reset(&mut self) {
    self.attempt = 0;
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Mutex;

  use super::*;

  #[derive(Default)]
  struct RecordingSleeper {
    delays: Mutex<Vec<Duration>>,
  }

  impl RecordingSleeper {
    fn delays(&self) -> Vec<Duration> {
      self.delays.lock().unwrap().clone()
    }
  }

  #[async_trait]
  impl Sleeper for RecordingSleeper {
    async fn sleep(&self, delay: Duration) {
      self.delays.lock().unwrap().push(delay);
    }
  }

  fn identity_jitter(delay: Duration) -> Duration {
    delay
  }

  #[test]
  fn should_grow_and_cap_the_backoff_then_reset() {
    let mut backoff = Backoff::with_jitter(
      Duration::from_millis(100),
      Duration::from_millis(500),
      Arc::new(identity_jitter),
    );
    assert_eq!(Duration::from_millis(100), backoff.next_delay());
    assert_eq!(Duration::from_millis(200), backoff.next_delay());
    assert_eq!(Duration::from_millis(400), backoff.next_delay());
    assert_eq!(Duration::from_millis(500), backoff.next_delay());
    assert_eq!(Duration::from_millis(500), backoff.next_delay());
    backoff.reset();
    assert_eq!(Duration::from_millis(100), backoff.next_delay());
  }

  #[tokio::test]
  async fn should_retry_until_success() {
    let sleeper = Arc::new(RecordingSleeper::default());
    let policy = RetryPolicy::new(
      5,
      Duration::from_secs(10),
      Duration::from_millis(10),
      Duration::from_millis(50),
    )
    .with_sleeper(sleeper.clone())
    .with_jitter(Arc::new(identity_jitter));
    let mut calls = 0;
    let result = policy
      .run(
        || {
          calls += 1;
          let calls = calls;
          async move {
            if calls < 3 {
              Err("transient")
            } else {
              Ok(calls)
            }
          }
        },
        |_| true,
      )
      .await;
    assert_eq!(3, result.unwrap());
    assert_eq!(
      vec![Duration::from_millis(10), Duration::from_millis(20)],
      sleeper.delays()
    );
  }

  #[tokio::test]
  async fn should_stop_at_max_attempts() {
    let policy = RetryPolicy::new(
      3,
      Duration::from_secs(10),
      Duration::from_millis(1),
      Duration::from_millis(2),
    )
    .with_sleeper(Arc::new(RecordingSleeper::default()))
    .with_jitter(Arc::new(identity_jitter));
    let mut calls = 0;
    let error = policy
      .run(
        || {
          calls += 1;
          async move { Err::<(), &str>("transient") }
        },
        |_| true,
      )
      .await
      .unwrap_err();
    assert_eq!(3, calls);
    assert_eq!(3, error.attempts);
    assert!(!error.permanent);
  }

  #[tokio::test]
  async fn should_not_retry_a_permanent_failure() {
    let policy = RetryPolicy::new(
      5,
      Duration::from_secs(10),
      Duration::from_millis(1),
      Duration::from_millis(2),
    );
    let mut calls = 0;
    let error = policy
      .run(
        || {
          calls += 1;
          async move { Err::<(), &str>("permanent") }
        },
        |_| false,
      )
      .await
      .unwrap_err();
    assert_eq!(1, calls);
    assert!(error.permanent);
  }

  #[tokio::test]
  async fn should_stop_when_the_elapsed_budget_is_exceeded() {
    let policy = RetryPolicy::new(
      10,
      Duration::from_millis(5),
      Duration::from_millis(10),
      Duration::from_millis(10),
    )
    .with_jitter(Arc::new(identity_jitter));
    let mut calls = 0;
    let error = policy
      .run(
        || {
          calls += 1;
          async move { Err::<(), &str>("transient") }
        },
        |_| true,
      )
      .await
      .unwrap_err();
    // The first delay would take the wall clock past the 5ms budget.
    assert_eq!(1, calls);
    assert!(!error.permanent);
  }
}
