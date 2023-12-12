/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Godot engine classes and methods.

// Re-exports of generated symbols
use crate::builtin::{GString, NodePath};
use crate::obj::dom::EngineDomain;
use crate::obj::{Gd, GodotClass, Inherits, InstanceId};

pub use crate::gen::central::global;
pub use crate::gen::classes::*;
pub use crate::gen::utilities;

use crate::sys;

mod io;
mod script_instance;

pub use io::*;
pub use script_instance::{create_script_instance, ScriptInstance};

/// Support for Godot _native structures_.
///
/// Native structures are a niche API in Godot. These are low-level data types that are passed as pointers to/from the engine.
/// In Rust, they are represented as `#[repr(C)]` structs.
///
/// There is unfortunately not much official documentation available; you may need to look at Godot source code.
/// Most users will not need native structures, as they are very specialized.
pub mod native {
    pub use crate::gen::native::*;
}

/// Extension trait for convenience functions on `PackedScene`
pub trait PackedSceneExt {
    /// ⚠️ Instantiates the scene as type `T`, panicking if not found or bad type.
    ///
    /// # Panics
    /// If the scene is not type `T` or inherited.
    fn instantiate_as<T>(&self) -> Gd<T>
    where
        T: Inherits<Node>,
    {
        self.try_instantiate_as::<T>()
            .unwrap_or_else(|| panic!("Failed to instantiate {to}", to = T::class_name()))
    }

    /// Instantiates the scene as type `T` (fallible).
    ///
    /// If the scene is not type `T` or inherited.
    fn try_instantiate_as<T>(&self) -> Option<Gd<T>>
    where
        T: Inherits<Node>;
}

impl PackedSceneExt for PackedScene {
    fn try_instantiate_as<T>(&self) -> Option<Gd<T>>
    where
        T: Inherits<Node>,
    {
        self.instantiate().and_then(|gd| gd.try_cast::<T>().ok())
    }
}

/// Extension trait with convenience functions for the node tree.
pub trait NodeExt {
    /// Retrieves the node at path `path`, panicking if not found or bad type.
    ///
    /// # Panics
    /// If the node is not found, or if it does not have type `T` or inherited.
    fn get_node_as<T>(&self, path: impl Into<NodePath>) -> Gd<T>
    where
        T: GodotClass + Inherits<Node>,
    {
        let path = path.into();
        let copy = path.clone(); // TODO avoid copy

        self.try_get_node_as(path).unwrap_or_else(|| {
            panic!(
                "There is no node of type {ty} path `{copy}`",
                ty = T::class_name()
            )
        })
    }

    /// Retrieves the node at path `path` (fallible).
    ///
    /// If the node is not found, or if it does not have type `T` or inherited,
    /// `None` will be returned.
    fn try_get_node_as<T>(&self, path: impl Into<NodePath>) -> Option<Gd<T>>
    where
        T: GodotClass + Inherits<Node>;
}

impl NodeExt for Node {
    fn try_get_node_as<T>(&self, path: impl Into<NodePath>) -> Option<Gd<T>>
    where
        T: GodotClass + Inherits<Node>,
    {
        let path = path.into();

        // TODO differentiate errors (not found, bad type) with Result
        self.get_node_or_null(path)
            .and_then(|node| node.try_cast::<T>().ok())
    }
}

impl<U> NodeExt for Gd<U>
where
    U: GodotClass<Declarer = EngineDomain> + Inherits<Node>,
{
    fn try_get_node_as<T>(&self, path: impl Into<NodePath>) -> Option<Gd<T>>
    where
        T: GodotClass + Inherits<Node>,
    {
        // TODO this could be implemented without share(), but currently lacks the proper bounds
        // This would need more sophisticated upcast design, e.g. T::upcast_{ref|mut}::<U>() for indirect relations
        // to make the indirect Deref more explicit

        let path = path.into();
        let node = self.clone().upcast::<Node>();

        <Node as NodeExt>::try_get_node_as(&*node, path)
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Utilities for crate

pub(crate) fn debug_string<T: GodotClass>(
    obj: &Gd<T>,
    f: &mut std::fmt::Formatter<'_>,
    ty: &str,
) -> std::fmt::Result {
    if let Some(id) = obj.instance_id_or_none() {
        let class: GString = obj.raw.as_object(|obj| Object::get_class(obj));
        write!(f, "{ty} {{ id: {id}, class: {class} }}")
    } else {
        write!(f, "{ty} {{ freed obj }}")
    }
}

pub(crate) fn display_string<T: GodotClass>(
    obj: &Gd<T>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let string: GString = obj.raw.as_object(Object::to_string);
    <GString as std::fmt::Display>::fmt(&string, f)
}

pub(crate) fn object_ptr_from_id(instance_id: InstanceId) -> sys::GDExtensionObjectPtr {
    // SAFETY: Godot looks up ID in ObjectDB and returns null if not found.
    unsafe { sys::interface_fn!(object_get_instance_from_id)(instance_id.to_u64()) }
}

pub(crate) fn ensure_object_alive(
    instance_id: Option<InstanceId>,
    old_object_ptr: sys::GDExtensionObjectPtr,
    method_name: &'static str,
) {
    let Some(instance_id) = instance_id else {
        panic!("{method_name}: cannot call method on null object")
    };

    let new_object_ptr = object_ptr_from_id(instance_id);

    assert!(
        !new_object_ptr.is_null(),
        "{method_name}: access to instance with ID {instance_id} after it has been freed"
    );

    // This should not happen, as reuse of instance IDs was fixed according to https://github.com/godotengine/godot/issues/32383,
    // namely in PR https://github.com/godotengine/godot/pull/36189. Double-check to make sure.
    assert_eq!(
        new_object_ptr, old_object_ptr,
        "{method_name}: instance ID {instance_id} points to a stale, reused object. Please report this to gdext maintainers."
    );
}
