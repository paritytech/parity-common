// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Tokio Runtime wrapper.

use futures::{future, Future, IntoFuture};
use std::sync::mpsc;
use std::{fmt, thread};
pub use tokio::runtime::{Builder as TokioRuntimeBuilder, Runtime as TokioRuntime, TaskExecutor};
pub use tokio::timer::Delay;

/// Runtime for futures.
///
/// Runs in a separate thread.
pub struct Runtime {
	executor: Executor,
	handle: RuntimeHandle,
}

impl Runtime {
	fn new(runtime_bldr: &mut TokioRuntimeBuilder) -> Self {
		let mut runtime = runtime_bldr.build().expect(
			"Building a Tokio runtime will only fail when mio components \
			 cannot be initialized (catastrophic)",
		);
		let (stop, stopped) = futures::oneshot();
		let (tx, rx) = mpsc::channel();
		let handle = thread::spawn(move || {
			tx.send(runtime.executor()).expect("Rx is blocking upper thread.");
			runtime
				.block_on(futures::empty().select(stopped).map(|_| ()).map_err(|_| ()))
				.expect("Tokio runtime should not have unhandled errors.");
		});
		let executor = rx.recv().expect("tx is transfered to a newly spawned thread.");

		Runtime {
			executor: Executor { inner: Mode::Tokio(executor) },
			handle: RuntimeHandle { close: Some(stop), handle: Some(handle) },
		}
	}

	/// Spawns a new tokio runtime with a default thread count on a background
	/// thread and returns a `Runtime` which can be used to spawn tasks via
	/// its executor.
	pub fn with_default_thread_count() -> Self {
		let mut runtime_bldr = TokioRuntimeBuilder::new();
		Self::new(&mut runtime_bldr)
	}

	/// Spawns a new tokio runtime with a the specified thread count on a
	/// background thread and returns a `Runtime` which can be used to spawn
	/// tasks via its executor.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn with_thread_count(thread_count: usize) -> Self {
		let mut runtime_bldr = TokioRuntimeBuilder::new();
		runtime_bldr.core_threads(thread_count);

		Self::new(&mut runtime_bldr)
	}

	/// Returns this runtime raw executor.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn raw_executor(&self) -> TaskExecutor {
		if let Mode::Tokio(ref executor) = self.executor.inner {
			executor.clone()
		} else {
			panic!("Runtime is not initialized in Tokio mode.")
		}
	}

	/// Returns runtime executor.
	pub fn executor(&self) -> Executor {
		self.executor.clone()
	}
}

#[derive(Clone)]
enum Mode {
	Tokio(TaskExecutor),
	// Mode used in tests
	#[allow(dead_code)]
	Sync,
	// Mode used in tests
	#[allow(dead_code)]
	ThreadPerFuture,
}

impl fmt::Debug for Mode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use self::Mode::*;

		match *self {
			Tokio(_) => write!(fmt, "tokio"),
			Sync => write!(fmt, "synchronous"),
			ThreadPerFuture => write!(fmt, "thread per future"),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Executor {
	inner: Mode,
}

impl Executor {
	/// Synchronous executor, used for tests.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_sync() -> Self {
		Executor { inner: Mode::Sync }
	}

	/// Spawns a new thread for each future (use only for tests).
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_thread_per_future() -> Self {
		Executor { inner: Mode::ThreadPerFuture }
	}

	/// Spawn a future on this runtime
	pub fn spawn<R>(&self, r: R)
	where
		R: IntoFuture<Item = (), Error = ()> + Send + 'static,
		R::Future: Send + 'static,
	{
		match self.inner {
			Mode::Tokio(ref executor) => executor.spawn(r.into_future()),
			Mode::Sync => {
				let _ = r.into_future().wait();
			}
			Mode::ThreadPerFuture => {
				thread::spawn(move || {
					let _ = r.into_future().wait();
				});
			}
		}
	}
}

impl<F: Future<Item = (), Error = ()> + Send + 'static> future::Executor<F> for Executor {
	fn execute(&self, future: F) -> Result<(), future::ExecuteError<F>> {
		match self.inner {
			Mode::Tokio(ref executor) => executor.execute(future),
			Mode::Sync => {
				let _ = future.wait();
				Ok(())
			}
			Mode::ThreadPerFuture => {
				thread::spawn(move || {
					let _ = future.wait();
				});
				Ok(())
			}
		}
	}
}

/// A handle to a runtime. Dropping the handle will cause runtime to shutdown.
pub struct RuntimeHandle {
	close: Option<futures::sync::oneshot::Sender<()>>,
	handle: Option<thread::JoinHandle<()>>,
}

impl From<Runtime> for RuntimeHandle {
	fn from(el: Runtime) -> Self {
		el.handle
	}
}

impl Drop for RuntimeHandle {
	fn drop(&mut self) {
		self.close.take().map(|v| v.send(()));
	}
}

impl RuntimeHandle {
	/// Blocks current thread and waits until the runtime is finished.
	pub fn wait(mut self) -> thread::Result<()> {
		self.handle.take().expect("Handle is taken only in `wait`, `wait` is consuming; qed").join()
	}

	/// Finishes this runtime.
	pub fn close(mut self) {
		let _ =
			self.close.take().expect("Close is taken only in `close` and `drop`. `close` is consuming; qed").send(());
	}
}
