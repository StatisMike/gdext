/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use crate::builtin::GString;
use crate::engine::global::Error;
use crate::gen::classes::{Resource, ResourceLoader, ResourceSaver};
use crate::obj::{Gd, GodotClass, Inherits};

use super::errors::GdIoError;

/// Loads a resource from the filesystem located at `path`, panicking on error.
///
/// See [`try_load`] for more information.
///
/// # Example
///
/// ```no_run
/// use godot::prelude::*;
///
/// let scene = load::<PackedScene>("res://path/to/Main.tscn");
/// ```
///
/// # Panics
/// If the resource cannot be loaded, or is not of type `T` or inherited.
#[inline]
pub fn load<T>(path: impl Into<GString>) -> Gd<T>
where
    T: GodotClass + Inherits<Resource>,
{
    let path = path.into();
    load_impl(&path).unwrap_or_else(|_| panic!("failed to load resource at path `{path}`"))
}

/// Loads a resource from the filesystem located at `path`.
///
/// The resource is loaded on the method call (unless it's referenced already elsewhere, e.g. in another script or in the scene),
/// which might cause slight delay, especially when loading scenes.
///
/// If the resource cannot be loaded, or is not of type `T` or inherited, this method returns `None`.
///
/// This method is a simplified version of [`ResourceLoader::load()`][crate::engine::ResourceLoader::load],
/// which can be used for more advanced scenarios.
///
/// # Note:
/// Resource paths can be obtained by right-clicking on a resource in the Godot editor (_FileSystem_ dock) and choosing "Copy Path",
/// or by dragging the file from the _FileSystem_ dock into the script.
///
/// The path must be absolute (typically starting with `res://`), a local path will fail.
///
/// # Example
/// Loads a scene called `Main` located in the `path/to` subdirectory of the Godot project and caches it in a variable.
/// The resource is directly stored with type `PackedScene`.
///
/// ```no_run
/// use godot::prelude::*;
///
/// if let Some(scene) = try_load::<PackedScene>("res://path/to/Main.tscn") {
///     // all good
/// } else {
///     // handle error
/// }
/// ```
#[inline]
pub fn try_load<T>(path: impl Into<GString>) -> Result<Gd<T>, GdIoError>
where
    T: GodotClass + Inherits<Resource>,
{
    load_impl(&path.into())
}

/// Saves a [`Resource`]-inheriting [`GodotClass`] `obj` into file located at `path`.
///
/// See [`try_save`] for more information.
///
/// # Panics
/// If the resouce cannot be saved.
pub fn save<T>(obj: Gd<T>, path: impl Into<GString>)
where
    T: GodotClass + Inherits<Resource>,
{
    let path = path.into();
    save_impl(obj, &path)
        .unwrap_or_else(|err| panic!("failed to save resource at path '{}': {}", &path, err));
}

/// Saves a [Resource]-inheriting [GodotClass] `obj` into file located at `path`.
///
/// This method is a simplified version of [`ResourceSaver::save()`][crate::engine::ResourceSaver::save] which can be used
/// for more advances scenarios.
///
/// ## Errors
///
pub fn try_save<T>(obj: Gd<T>, path: impl Into<GString>) -> Result<(), GdIoError>
where
    T: GodotClass + Inherits<Resource>,
{
    save_impl(obj, &path.into())
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Implementations of this file

// Separate functions, to avoid constructing string twice
// Note that more optimizations than that likely make no sense, as loading is quite expensive
fn load_impl<T>(path: &GString) -> Result<Gd<T>, GdIoError>
where
    T: GodotClass + Inherits<Resource>,
{
    // TODO unclone GString
    match ResourceLoader::singleton()
        .load_ex(path.clone())
        .type_hint(T::class_name().to_godot_string())
        .done()
    {
        Some(res) => match res.try_cast::<T>() {
            Ok(obj) => Ok(obj),
            Err(gd) => Err(GdIoError::ResourceCantCast(gd)),
        },
        None => Err(GdIoError::ResourceNotFound),
    }
}

fn save_impl<T>(obj: Gd<T>, path: &GString) -> Result<(), GdIoError>
where
    T: GodotClass + Inherits<Resource>,
{
    let res = ResourceSaver::singleton()
        .save_ex(obj.upcast())
        .path(path.clone())
        .done();

    if res == Error::OK {
        return Ok(());
    }
    Err(GdIoError::ResourceSave(res))
}
