/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

mod errors;
mod gfile;
mod resource;

pub use errors::{GdIoError, NotUniqueError};
pub use gfile::GFile;
pub use resource::{load, save, try_load, try_save};
