//! Success, failure, and forward handling.
//!
//! The `Outcome<S, E, F>` type is similar to the standard library's `Result<S,
//! E>` type. It is an enum with three variants, each containing a value:
//! `Success(S)`, which represents a successful outcome, `Failure(E)`, which
//! represents a failing outcome, and `Forward(F)`, which represents neither a
//! success or failure, but instead, indicates that processing could not be
//! handled and should instead be _forwarded_ to whatever can handle the
//! processing next.
//!
//! The `Outcome` type is the return type of many of the core Rocket traits,
//! including [FromRequest](/rocket/request/trait.FromRequest.html),
//! [FromData](/rocket/data/trait.FromData.html), and
//! [Responder](/rocket/response/trait.Responder.html). It is also the return
//! type of request handlers via the
//! [Response](/rocket/response/struct.Response.html) type.
//!
//! # Success
//!
//! A successful `Outcome<S, E, F>`, `Success(S)`, is returned from functions
//! that complete successfully. The meaning of a `Success` outcome depends on
//! the context. For instance, the `Outcome` of the `from_data` method of the
//! `FromData` trait will be matched against the type expected by the user. For
//! example, consider the following handler:
//!
//! ```rust,ignore
//! #[post("/", data = "<my_val>")]
//! fn hello(my_val: S) -> ... {  }
//! ```
//!
//! The `FromData` implementation for the type `S` returns an `Outcome` with a
//! `Success(S)`. If `from_data` returns a `Success`, the `Success` value will
//! be unwrapped and the value will be used as the value of `my_val`.
//!
//! # Failure
//!
//! A failure `Outcome<S, E, F>`, `Failure(E)`, is returned when a function
//! fails with some error and no processing can or should continue as a result.
//! The meaning of a failure depends on the context.
//!
//! It Rocket, a `Failure` generally means that a request is taken out of normal
//! processing. The request is then given to the catcher corresponding to some
//! status code. users can catch failures by requesting a type of `Result<S, E>`
//! or `Option<S>` in request handlers. For example, if a user's handler looks
//! like:
//!
//! ```rust,ignore
//! #[post("/", data = "<my_val>")]
//! fn hello(my_val: Result<S, E>) -> ... {  }
//! ```
//!
//! The `FromData` implementation for the type `S` returns an `Outcome` with a
//! `Success(S)` and `Failure(E)`. If `from_data` returns a `Failure`, the
//! `Failure` value will be unwrapped and the value will be used as the `Err`
//! value of `my_val` while a `Success` will be unwrapped and used the `Ok`
//! value.
//!
//! # Forward
//!
//! A forward `Outcome<S, E, F>`, `Forward(F)`, is returned when a function
//! wants to indicate that the requested processing should be _forwarded_ to the
//! next available processor. Again, the exact meaning depends on the context.
//!
//! In Rocket, a `Forward` generally means that a request is forwarded to the
//! next available request handler. For example, consider the following request
//! handler:
//!
//! ```rust,ignore
//! #[post("/", data = "<my_val>")]
//! fn hello(my_val: S) -> ... {  }
//! ```
//!
//! The `FromData` implementation for the type `S` returns an `Outcome` with a
//! `Success(S)`, `Failure(E)`, and `Forward(F)`. If the `Outcome` is a
//! `Forward`, the `hello` handler isn't called. Instead, the incoming request
//! is forwarded, or passed on to, the next matching route, if any. Ultimately,
//! if there are no non-forwarding routes, forwarded requests are handled by the
//! 404 catcher. Similar to `Failure`s, users can catch `Forward`s by requesting
//! a type of `Option<S>`. If an `Outcome` is a `Forward`, the `Option` will be
//! `None`.

use std::fmt;

use term_painter::Color::*;
use term_painter::Color;
use term_painter::ToStyle;

use self::Outcome::*;

/// An enum representing success (`Success`), failure (`Failure`), or
/// forwarding (`Forward`).
///
/// See the [top level documentation](/rocket/outcome/) for detailed
/// information.
#[must_use]
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Outcome<S, E, F> {
    /// Contains the success value.
    Success(S),
    /// Contains the failure error value.
    Failure(E),
    /// Contains the value to forward on.
    Forward(F),
}

/// Conversion trait from some type into an Outcome type.
pub trait IntoOutcome<S, E, F> {
    fn into_outcome(self) -> Outcome<S, E, F>;
}

impl<S, E, F> Outcome<S, E, F> {
    /// Unwraps the Outcome, yielding the contents of a Success.
    ///
    /// # Panics
    ///
    /// Panics if the value is not `Success`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rocket::outcome::Outcome;
    /// # use rocket::outcome::Outcome::*;
    /// #
    /// let x: Outcome<i32, &str, usize> = Success(10);
    /// assert_eq!(x.unwrap(), 10);
    /// ```
    #[inline]
    pub fn unwrap(self) -> S {
        match self {
            Success(val) => val,
            _ => panic!("Expected a successful outcome!")
        }
    }

    /// Unwraps the Outcome, yielding the contents of a Success.
    ///
    /// # Panics
    ///
    /// If the value is not `Success`, panics with the given `message`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rocket::outcome::Outcome;
    /// # use rocket::outcome::Outcome::*;
    /// #
    /// let x: Outcome<i32, &str, usize> = Success(10);
    /// assert_eq!(x.expect("success value"), 10);
    /// ```
    #[inline]
    pub fn expect(self, message: &str) -> S {
        match self {
            Success(val) => val,
            _ => panic!("Outcome::expect() failed: {}", message)
        }
    }

    /// Return true if this `Outcome` is a `Success`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rocket::outcome::Outcome;
    /// # use rocket::outcome::Outcome::*;
    /// #
    /// let x: Outcome<i32, &str, usize> = Success(10);
    /// assert_eq!(x.is_success(), true);
    ///
    /// let x: Outcome<i32, &str, usize> = Failure("Hi! I'm an error.");
    /// assert_eq!(x.is_success(), false);
    ///
    /// let x: Outcome<i32, &str, usize> = Forward(25);
    /// assert_eq!(x.is_success(), false);
    /// ```
    #[inline]
    pub fn is_success(&self) -> bool {
        match *self {
            Success(_) => true,
            _ => false
        }
    }

    /// Return true if this `Outcome` is a `Failure`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rocket::outcome::Outcome;
    /// # use rocket::outcome::Outcome::*;
    /// #
    /// let x: Outcome<i32, &str, usize> = Success(10);
    /// assert_eq!(x.is_failure(), false);
    ///
    /// let x: Outcome<i32, &str, usize> = Failure("Hi! I'm an error.");
    /// assert_eq!(x.is_failure(), true);
    ///
    /// let x: Outcome<i32, &str, usize> = Forward(25);
    /// assert_eq!(x.is_failure(), false);
    /// ```
    #[inline]
    pub fn is_failure(&self) -> bool {
        match *self {
            Failure(_) => true,
            _ => false
        }
    }

    /// Return true if this `Outcome` is a `Forward`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rocket::outcome::Outcome;
    /// # use rocket::outcome::Outcome::*;
    /// #
    /// let x: Outcome<i32, &str, usize> = Success(10);
    /// assert_eq!(x.is_forward(), false);
    ///
    /// let x: Outcome<i32, &str, usize> = Failure("Hi! I'm an error.");
    /// assert_eq!(x.is_forward(), false);
    ///
    /// let x: Outcome<i32, &str, usize> = Forward(25);
    /// assert_eq!(x.is_forward(), true);
    /// ```
    #[inline]
    pub fn is_forward(&self) -> bool {
        match *self {
            Forward(_) => true,
            _ => false
        }
    }

    /// Converts from `Outcome<S, E, F>` to `Option<S>`.
    ///
    /// Returns the `Some` of the `Success` if this is a `Success`, otherwise
    /// returns `None`. `self` is consumed, and all other values are discarded.
    ///
    /// ```rust
    /// # use rocket::outcome::Outcome;
    /// # use rocket::outcome::Outcome::*;
    /// #
    /// let x: Outcome<i32, &str, usize> = Success(10);
    /// assert_eq!(x.succeeded(), Some(10));
    ///
    /// let x: Outcome<i32, &str, usize> = Failure("Hi! I'm an error.");
    /// assert_eq!(x.succeeded(), None);
    ///
    /// let x: Outcome<i32, &str, usize> = Forward(25);
    /// assert_eq!(x.succeeded(), None);
    /// ```
    #[inline]
    pub fn succeeded(self) -> Option<S> {
        match self {
            Success(val) => Some(val),
            _ => None
        }
    }

    /// Converts from `Outcome<S, E, F>` to `Option<E>`.
    ///
    /// Returns the `Some` of the `Failure` if this is a `Failure`, otherwise
    /// returns `None`. `self` is consumed, and all other values are discarded.
    ///
    /// ```rust
    /// # use rocket::outcome::Outcome;
    /// # use rocket::outcome::Outcome::*;
    /// #
    /// let x: Outcome<i32, &str, usize> = Success(10);
    /// assert_eq!(x.failed(), None);
    ///
    /// let x: Outcome<i32, &str, usize> = Failure("Hi! I'm an error.");
    /// assert_eq!(x.failed(), Some("Hi! I'm an error."));
    ///
    /// let x: Outcome<i32, &str, usize> = Forward(25);
    /// assert_eq!(x.failed(), None);
    /// ```
    #[inline]
    pub fn failed(self) -> Option<E> {
        match self {
            Failure(val) => Some(val),
            _ => None
        }
    }

    /// Converts from `Outcome<S, E, F>` to `Option<F>`.
    ///
    /// Returns the `Some` of the `Forward` if this is a `Forward`, otherwise
    /// returns `None`. `self` is consumed, and all other values are discarded.
    ///
    /// ```rust
    /// # use rocket::outcome::Outcome;
    /// # use rocket::outcome::Outcome::*;
    /// #
    /// let x: Outcome<i32, &str, usize> = Success(10);
    /// assert_eq!(x.forwarded(), None);
    ///
    /// let x: Outcome<i32, &str, usize> = Failure("Hi! I'm an error.");
    /// assert_eq!(x.forwarded(), None);
    ///
    /// let x: Outcome<i32, &str, usize> = Forward(25);
    /// assert_eq!(x.forwarded(), Some(25));
    /// ```
    #[inline]
    pub fn forwarded(self) -> Option<F> {
        match self {
            Forward(val) => Some(val),
            _ => None
        }
    }

    /// Converts from `Outcome<S, E, F>` to `Outcome<&S, &E, &F>`.
    ///
    /// ```rust
    /// # use rocket::outcome::Outcome;
    /// # use rocket::outcome::Outcome::*;
    /// #
    /// let x: Outcome<i32, &str, usize> = Success(10);
    /// assert_eq!(x.as_ref(), Success(&10));
    ///
    /// let x: Outcome<i32, &str, usize> = Failure("Hi! I'm an error.");
    /// assert_eq!(x.as_ref(), Failure(&"Hi! I'm an error."));
    /// ```
    #[inline]
    pub fn as_ref(&self) -> Outcome<&S, &E, &F> {
        match *self {
            Success(ref val) => Success(val),
            Failure(ref val) => Failure(val),
            Forward(ref val) => Forward(val),
        }
    }

    /// Converts from `Outcome<S, E, F>` to `Outcome<&mut S, &mut E, &mut F>`.
    ///
    /// ```rust
    /// # use rocket::outcome::Outcome;
    /// # use rocket::outcome::Outcome::*;
    /// #
    /// let mut x: Outcome<i32, &str, usize> = Success(10);
    /// if let Success(val) = x.as_mut() {
    ///     *val = 20;
    /// }
    ///
    /// assert_eq!(x.unwrap(), 20);
    /// ```
    #[inline]
    pub fn as_mut(&mut self) -> Outcome<&mut S, &mut E, &mut F> {
        match *self {
            Success(ref mut val) => Success(val),
            Failure(ref mut val) => Failure(val),
            Forward(ref mut val) => Forward(val),
        }
    }

    #[inline]
    fn formatting(&self) -> (Color, &'static str) {
        match *self {
            Success(..) => (Green, "Success"),
            Failure(..) => (Red, "Failure"),
            Forward(..) => (Yellow, "Forward"),
        }
    }
}

impl<S, E, F> fmt::Debug for Outcome<S, E, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Outcome::{}", self.formatting().1)
    }
}

impl<S, E, F> fmt::Display for Outcome<S, E, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (color, string) = self.formatting();
        write!(f, "{}", color.paint(string))
    }
}
