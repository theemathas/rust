// Test when the captured `&Location` in a future is aliased with a `&mut`

#![feature(async_fn_track_caller)]

use std::future::{Future, poll_fn};
use std::panic::Location;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

// Creates a future whose poll() returns Pending once, then returns Ready
fn pending_once() -> impl Future<Output = ()> {
    let mut is_ready = false;
    poll_fn(move |_| {
        if is_ready {
            Poll::Ready(())
        } else {
            is_ready = true;
            Poll::Pending
        }
    })
}

#[track_caller]
async fn foo() {
    pending_once().await;
    // Passing the invalidated `&Location` as an implicit parameter is fine.
    bar();
    // Do the UB call as a separate call so it's obvious where the UB is.
    baz();
    //~^ERROR: that tag does not exist in the borrow stack for this location
}

#[track_caller]
fn bar() {}

#[track_caller]
fn baz() {
    // Accessing the invalidated implicit parameter is UB
    let _ = Location::caller();
}

fn main() {
    let mut future = foo();
    // Initialize a `Location` with some dummy value.
    let mut fake_caller_location = *Location::caller();
    // The future has the captured caller location followed by a discriminant.
    assert_eq!(size_of_val(&future), 2 * size_of::<*const ()>());
    unsafe {
        // Make the captured caller location (`&Location`) point to memory we control.
        (&raw mut future).cast::<*const Location>().write(&raw const fake_caller_location);
    }
    let mut future = unsafe { Pin::new_unchecked(&mut future) };
    let mut cx = Context::from_waker(Waker::noop());

    // Stop at the pending_once().
    assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
    // Invalidate the captured `&Location` in the future.
    let _ = &mut fake_caller_location;
    let _ = future.as_mut().poll(&mut cx);
}
