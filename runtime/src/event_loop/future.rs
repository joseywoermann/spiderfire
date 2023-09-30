/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::RefCell;
use std::task;
use std::task::Poll;

use futures::stream::FuturesUnordered;
use futures::StreamExt;
use mozjs::jsapi::JSObject;
use tokio::task::JoinHandle;

use ion::{Context, Error, ErrorKind, ErrorReport, Promise, ThrowException, Value};
use ion::conversions::BoxedIntoValue;

type FutureOutput = (Result<BoxedIntoValue, BoxedIntoValue>, *mut JSObject);

#[derive(Default)]
pub struct FutureQueue {
	queue: RefCell<FuturesUnordered<JoinHandle<FutureOutput>>>,
}

impl FutureQueue {
	pub fn run_futures(&self, cx: &Context, wcx: &mut task::Context) -> Result<(), Option<ErrorReport>> {
		let mut results = Vec::new();

		let mut queue = self.queue.borrow_mut();
		while let Poll::Ready(Some(item)) = queue.poll_next_unpin(wcx) {
			match item {
				Ok(item) => results.push(item),
				Err(error) => {
					Error::new(&error.to_string(), ErrorKind::Normal).throw(cx);
					return Err(None);
				}
			}
		}

		for (result, promise) in results {
			let mut value = Value::undefined(cx);
			let promise = Promise::from(cx.root_object(promise)).unwrap();

			let result = match result {
				Ok(o) => unsafe {
					o.into_value(cx, &mut value);
					promise.resolve(cx, &value)
				},
				Err(e) => unsafe {
					e.into_value(cx, &mut value);
					promise.reject(cx, &value)
				},
			};

			if !result {
				return Err(ErrorReport::new_with_exception_stack(cx));
			}
		}

		Ok(())
	}

	pub fn enqueue(&self, handle: JoinHandle<FutureOutput>) {
		self.queue.borrow().push(handle);
	}

	pub fn is_empty(&self) -> bool {
		self.queue.borrow().is_empty()
	}
}
