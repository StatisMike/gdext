/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use crate::engine::global::Error;
use crate::gen::classes::{RefCounted, Resource};
use crate::obj::{Gd, Inherits};

/// Error during I-O operations
#[derive(Debug)]
pub enum GdIoError {
    ResourceNotFound,
    ResourceCantCast(Gd<Resource>),
    ResourceSave(Error),
    FileNotOpen,
    FileAccessReference(NotUniqueError),
}

impl std::fmt::Display for GdIoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => write!(f, "no ok"),
        }
    }
}

/// Error stemming from the non-uniqueness of the [`Gd`] instance.
///
/// Keeping track of the uniqueness of references can be crucial in many applications, especially if we want to ensure
/// that the passed [`Gd`] reference will be possessed by only one different object instance or function in its lifetime.
///
/// Only applicable to [`GodotClass`](crate::obj::GodotClass) objects that inherit from [`RefCounted`]. To check the
/// uniqueness, call the `check()` associated method.
///
/// ## Example
///
/// ```no_run
/// use godot::engine::RefCounted;
/// use godot::engine::NotUniqueError;
///
/// let shared = RefCounted::new();
/// let cloned = shared.clone();
/// let result = NotUniqueError::check(shared);
///
/// assert!(result.is_err());
///
/// if let Err(error) = result {
///     assert_eq!(error.get_reference_count(), 2)
/// }
/// ```
#[derive(Debug)]
pub struct NotUniqueError {
    reference_count: i32,
}

impl NotUniqueError {
    /// check [`Gd`] reference uniqueness.
    ///
    /// Checks the [`Gd`] of the [`GodotClass`](crate::obj::GodotClass) that inherits from [`RefCounted`] if it is unique
    /// reference to the object.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use godot::engine::RefCounted;
    /// use godot::engine::NotUniqueError;
    ///
    /// let unique = RefCounted::new();
    /// assert!(NotUniqueError::check(unique).is_ok());
    ///
    /// let shared = RefCounted::new();
    /// let cloned = shared.clone();
    /// assert!(NotUniqueError::check(shared).is_err());
    /// assert!(NotUniqueError::check(cloned).is_err());
    /// ```
    pub fn check<T>(rc: Gd<T>) -> Result<Gd<T>, Self>
    where
        T: Inherits<RefCounted>,
    {
        let rc = rc.upcast::<RefCounted>();
        let reference_count = rc.get_reference_count();

        if reference_count != 1 {
            Err(Self { reference_count })
        } else {
            Ok(rc.cast::<T>())
        }
    }

    /// Get the detected reference count
    pub fn get_reference_count(&self) -> i32 {
        self.reference_count
    }
}

impl std::fmt::Display for NotUniqueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pointer is not unique, current reference count: {}",
            self.reference_count
        )
    }
}
