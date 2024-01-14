/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO remove this warning once impl is complete
// Several types have #[allow(dead_code)], can be subsequently removed.

#![allow(clippy::question_mark)] // in #[derive(DeJson)]

use nanoserde::DeJson;

// ----------------------------------------------------------------------------------------------------------------------------------------------
// JSON models

#[derive(DeJson)]
pub struct JsonExtensionApi {
    pub header: JsonHeader,
    pub builtin_class_sizes: Vec<JsonClassSizes>,
    pub builtin_classes: Vec<JsonBuiltin>,
    pub classes: Vec<JsonClass>,
    pub global_enums: Vec<JsonEnum>,
    pub utility_functions: Vec<JsonUtilityFunction>,
    pub native_structures: Vec<JsonNativeStructure>,
    pub singletons: Vec<JsonSingleton>,
}

#[derive(DeJson, Clone, Debug)]
pub struct JsonHeader {
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
    pub version_status: String,
    pub version_build: String,
    pub version_full_name: String,
}

#[derive(DeJson)]
pub struct JsonClassSizes {
    pub build_configuration: String,
    pub sizes: Vec<JsonClassSize>,
}

#[derive(DeJson)]
pub struct JsonClassSize {
    pub name: String,
    pub size: usize,
}

#[derive(DeJson)]
pub struct JsonBuiltin {
    pub name: String,
    pub indexing_return_type: Option<String>,
    pub is_keyed: bool,
    // pub members: Option<Vec<Member>>,
    // pub constants: Option<Vec<BuiltinConstant>>,
    pub enums: Option<Vec<JsonBuiltinEnum>>, // no bitfield
    pub operators: Vec<JsonOperator>,
    pub methods: Option<Vec<JsonBuiltinMethod>>,
    pub constructors: Vec<JsonConstructor>,
    pub has_destructor: bool,
}

#[derive(DeJson)]
pub struct JsonClass {
    pub name: String,
    pub is_refcounted: bool,
    pub is_instantiable: bool,
    pub inherits: Option<String>,
    pub api_type: String,
    pub constants: Option<Vec<JsonClassConstant>>,
    pub enums: Option<Vec<JsonEnum>>,
    pub methods: Option<Vec<JsonClassMethod>>,
    // pub properties: Option<Vec<Property>>,
    // pub signals: Option<Vec<Signal>>,
}

#[derive(DeJson)]
pub struct JsonNativeStructure {
    pub name: String,
    pub format: String,
}

#[derive(DeJson)]
pub struct JsonSingleton {
    pub name: String,
    // Note: `type` currently has always same value as `name`, thus redundant
    // #[nserde(rename = "type")]
    // type_: String,
}

#[derive(DeJson)]
pub struct JsonEnum {
    pub name: String,
    pub is_bitfield: bool,
    pub values: Vec<JsonEnumConstant>,
}

#[derive(DeJson)]
pub struct JsonBuiltinEnum {
    pub name: String,
    pub values: Vec<JsonEnumConstant>,
}

impl JsonBuiltinEnum {
    pub(crate) fn to_enum(&self) -> JsonEnum {
        JsonEnum {
            name: self.name.clone(),
            is_bitfield: false,
            values: self.values.clone(),
        }
    }
}

#[derive(DeJson, Clone)]
pub struct JsonEnumConstant {
    pub name: String,

    // i64 is common denominator for enum, bitfield and constant values.
    // Note that values > i64::MAX will be implicitly wrapped, see https://github.com/not-fl3/nanoserde/issues/89.
    pub value: i64,
}

impl JsonEnumConstant {
    pub fn to_enum_ord(&self) -> i32 {
        self.value.try_into().unwrap_or_else(|_| {
            panic!(
                "enum value {} = {} is out of range for i32, please report this",
                self.name, self.value
            )
        })
    }
}

pub type JsonClassConstant = JsonEnumConstant;

/*
// Constants of builtin types have a string value like "Vector2(1, 1)", hence also a type field
#[derive(DeJson)]
pub struct JsonBuiltinConstant {
    pub name: String,
    #[nserde(rename = "type")]
    pub type_: String,
    pub value: String,
}
*/

#[derive(DeJson)]
pub struct JsonOperator {
    pub name: String,
    pub right_type: Option<String>, // null if unary
    pub return_type: String,
}

#[derive(DeJson)]
pub struct JsonMember {
    pub name: String,
    #[nserde(rename = "type")]
    pub type_: String,
}

#[derive(DeJson)]
#[allow(dead_code)]
pub struct JsonProperty {
    #[nserde(rename = "type")]
    type_: String,
    name: String,
    setter: String,
    getter: String,
    index: i32, // can be -1
}

#[derive(DeJson)]
#[allow(dead_code)]
pub struct JsonSignal {
    name: String,
    arguments: Option<Vec<JsonMethodArg>>,
}

#[derive(DeJson)]
pub struct JsonConstructor {
    pub index: usize,
    pub arguments: Option<Vec<JsonMethodArg>>,
}

#[derive(DeJson)]
pub struct JsonUtilityFunction {
    pub name: String,
    pub return_type: Option<String>,
    /// `"general"` or `"math"`
    pub category: String,
    pub is_vararg: bool,
    pub hash: i64,
    pub arguments: Option<Vec<JsonMethodArg>>,
}

#[derive(DeJson)]
pub struct JsonBuiltinMethod {
    pub name: String,
    pub return_type: Option<String>,
    pub is_vararg: bool,
    pub is_const: bool,
    pub is_static: bool,
    pub hash: Option<i64>,
    pub arguments: Option<Vec<JsonMethodArg>>,
}

#[derive(DeJson, Clone)]
pub struct JsonClassMethod {
    pub name: String,
    pub is_const: bool,
    pub is_vararg: bool,
    pub is_static: bool,
    pub is_virtual: bool,
    pub hash: Option<i64>,
    pub return_value: Option<JsonMethodReturn>,
    pub arguments: Option<Vec<JsonMethodArg>>,
}

// Example: set_point_weight_scale ->
// [ {name: "id", type: "int", meta: "int64"},
//   {name: "weight_scale", type: "float", meta: "float"},
#[derive(DeJson, Clone)]
pub struct JsonMethodArg {
    pub name: String,
    #[nserde(rename = "type")]
    pub type_: String,
    pub meta: Option<String>,
    pub default_value: Option<String>,
}

// Example: get_available_point_id -> {type: "int", meta: "int64"}
#[derive(DeJson, Clone)]
pub struct JsonMethodReturn {
    #[nserde(rename = "type")]
    pub type_: String,
    pub meta: Option<String>,
}

impl JsonMethodReturn {
    pub fn from_type_no_meta(type_: &str) -> Self {
        Self {
            type_: type_.to_owned(),
            meta: None,
        }
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Implementation

pub fn load_extension_api(
    watch: &mut godot_bindings::StopWatch,
) -> (JsonExtensionApi, [&'static str; 2]) {
    // For float/double inference, see:
    // * https://github.com/godotengine/godot-proposals/issues/892
    // * https://github.com/godotengine/godot-cpp/pull/728
    // Have to do target_pointer_width check after code generation
    // So pass a [32bit, 64bit] around of appropriate precision
    // For why see: https://github.com/rust-lang/rust/issues/42587
    let build_config: [&'static str; 2] = {
        if cfg!(feature = "double-precision") {
            ["double_32", "double_64"]
        } else {
            ["float_32", "float_64"]
        }
    };
    // Use type inference, so we can accept both String (dynamically resolved) and &str (prebuilt).
    // #[allow]: as_ref() acts as impl AsRef<str>, but with conditional compilation

    let json = godot_bindings::load_gdextension_json(watch);
    #[allow(clippy::useless_asref)]
    let json_str: &str = json.as_ref();

    let model: JsonExtensionApi =
        DeJson::deserialize_json(json_str).expect("failed to deserialize JSON");
    watch.record("deserialize_json");

    println!("Parsed extension_api.json for version {:?}", model.header);

    (model, build_config)
}
