use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

use crate::TantivyError;

/// `FutureResult` is a handle that makes it possible to wait for the completion
/// of an ongoing task.
///
/// Contrary to some `Future`, it does not need to be polled for the task to
/// progress. Dropping the `FutureResult` does not cancel the task being executed
/// either.
///
/// - In a sync context, you can call `FutureResult::wait()`. The function does not rely on
///   `block_on`.
/// - In an async context, you can call simply use `FutureResult` as a future.
pub struct FutureResult<T> {
    inner: Inner<T>,
}

enum Inner<T> {
    FailedBeforeStart(Option<TantivyError>),
    InProgress {
        receiver: ReceiverState<T>,
        error_msg_if_failure: &'static str,
    },
}

enum ReceiverState<T> {
    Blocking(oneshot::Receiver<crate::Result<T>>),
    Async(oneshot::AsyncReceiver<crate::Result<T>>),
    Consumed,
}

struct ThreadWake(std::thread::Thread);

impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

// A receiver already polled as a future cannot be converted back to a blocking
// receiver. Park the current thread using its async waker in that rare case.
fn wait_for_async<T>(mut receiver: oneshot::AsyncReceiver<T>) -> Result<T, oneshot::RecvError> {
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    loop {
        match Pin::new(&mut receiver).poll(&mut context) {
            Poll::Ready(result) => return result,
            Poll::Pending => std::thread::park(),
        }
    }
}

impl<T> From<TantivyError> for FutureResult<T> {
    fn from(err: TantivyError) -> Self {
        FutureResult {
            inner: Inner::FailedBeforeStart(Some(err)),
        }
    }
}

impl<T> FutureResult<T> {
    pub(crate) fn create(
        error_msg_if_failure: &'static str,
    ) -> (Self, oneshot::Sender<crate::Result<T>>) {
        let (sender, receiver) = oneshot::channel();
        let inner: Inner<T> = Inner::InProgress {
            receiver: ReceiverState::Blocking(receiver),
            error_msg_if_failure,
        };
        (FutureResult { inner }, sender)
    }

    /// Blocks until the scheduled result is available.
    ///
    /// In an async context, you should simply use `ScheduledResult` as a future.
    pub fn wait(self) -> crate::Result<T> {
        match self.inner {
            Inner::FailedBeforeStart(err) => Err(err.unwrap()),
            Inner::InProgress {
                receiver,
                error_msg_if_failure,
            } => match receiver {
                ReceiverState::Blocking(receiver) => receiver.recv(),
                ReceiverState::Async(receiver) => wait_for_async(receiver),
                ReceiverState::Consumed => panic!("future polled after completion"),
            }
            .unwrap_or_else(|_| {
                Err(crate::TantivyError::SystemError(
                    error_msg_if_failure.to_string(),
                ))
            }),
        }
    }
}

impl<T> Future for FutureResult<T> {
    type Output = crate::Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match &mut self.get_mut().inner {
            Inner::FailedBeforeStart(err) => Poll::Ready(Err(err.take().unwrap())),
            Inner::InProgress {
                receiver,
                error_msg_if_failure,
            } => {
                if matches!(receiver, ReceiverState::Blocking(_)) {
                    let ReceiverState::Blocking(blocking) =
                        std::mem::replace(receiver, ReceiverState::Consumed)
                    else {
                        unreachable!()
                    };
                    *receiver = ReceiverState::Async(blocking.into_future());
                }
                let ReceiverState::Async(receiver) = receiver else {
                    panic!("future polled after completion");
                };
                match Future::poll(Pin::new(receiver), cx) {
                    Poll::Ready(oneshot_res) => {
                        let res = oneshot_res.unwrap_or_else(|_| {
                            Err(crate::TantivyError::SystemError(
                                error_msg_if_failure.to_string(),
                            ))
                        });
                        Poll::Ready(res)
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};
    use std::time::Duration;

    use futures::executor::block_on;

    use super::FutureResult;
    use crate::TantivyError;

    #[test]
    fn test_scheduled_result_failed_to_schedule() {
        let scheduled_result: FutureResult<()> = FutureResult::from(TantivyError::Poisoned);
        let res = block_on(scheduled_result);
        assert!(matches!(res, Err(TantivyError::Poisoned)));
    }

    #[test]

    fn test_scheduled_result_error() {
        let (scheduled_result, tx): (FutureResult<()>, _) = FutureResult::create("failed");
        drop(tx);
        let res = block_on(scheduled_result);
        assert!(matches!(res, Err(TantivyError::SystemError(_))));
    }

    #[test]
    fn test_scheduled_result_sent_success() {
        let (scheduled_result, tx): (FutureResult<u64>, _) = FutureResult::create("failed");
        tx.send(Ok(2u64)).unwrap();
        assert_eq!(block_on(scheduled_result).unwrap(), 2u64);
    }

    #[test]
    fn test_scheduled_result_sent_error() {
        let (scheduled_result, tx): (FutureResult<u64>, _) = FutureResult::create("failed");
        tx.send(Err(TantivyError::Poisoned)).unwrap();
        let res = block_on(scheduled_result);
        assert!(matches!(res, Err(TantivyError::Poisoned)));
    }

    #[test]
    fn wait_without_poll_receives_value_and_sender_drop() {
        let (result, tx) = FutureResult::create("failed");
        tx.send(Ok(42)).unwrap();
        assert_eq!(result.wait().unwrap(), 42);

        let (result, tx): (FutureResult<()>, _) = FutureResult::create("sender dropped");
        drop(tx);
        assert!(matches!(result.wait(), Err(TantivyError::SystemError(_))));
    }

    #[test]
    fn wait_after_pending_poll_blocks_until_sent() {
        let (mut result, tx) = FutureResult::create("failed");
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Pin::new(&mut result).poll(&mut context),
            Poll::Pending
        ));
        let sender = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            tx.send(Ok(42)).unwrap();
        });
        assert_eq!(result.wait().unwrap(), 42);
        sender.join().unwrap();
    }

    #[test]
    fn wait_after_pending_poll_reports_sender_drop() {
        let (mut result, tx): (FutureResult<()>, _) = FutureResult::create("sender dropped");
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Pin::new(&mut result).poll(&mut context),
            Poll::Pending
        ));
        drop(tx);
        assert!(matches!(result.wait(), Err(TantivyError::SystemError(_))));
    }

    struct WakeCounter(AtomicUsize);

    impl Wake for WakeCounter {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn pending_future_is_woken_when_result_arrives() {
        let (mut result, tx) = FutureResult::create("failed");
        let counter = Arc::new(WakeCounter(AtomicUsize::new(0)));
        let waker = Waker::from(counter.clone());
        let mut context = Context::from_waker(&waker);
        assert!(matches!(
            Pin::new(&mut result).poll(&mut context),
            Poll::Pending
        ));
        tx.send(Ok(42)).unwrap();
        assert_eq!(counter.0.load(Ordering::SeqCst), 1);
        assert!(matches!(
            Pin::new(&mut result).poll(&mut context),
            Poll::Ready(Ok(42))
        ));
    }
}
