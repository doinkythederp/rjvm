#![cfg_attr(not(feature = "std"), no_std)]
#![feature(error_in_core)]

extern crate alloc;
extern crate zip_parser as zip;

pub mod abstract_object;
pub mod alloc_entry;
pub mod array;
pub mod array_entry_type;
mod call_frame;
pub mod call_stack;
pub mod class;
pub mod class_and_method;
mod class_loader;
mod class_manager;
mod class_path;
mod class_path_entry;
mod class_resolver_by_id;
pub mod exceptions;
mod file_system_class_path_entry;
mod gc;
pub mod io;
mod jar_file_class_path_entry;
pub mod java_objects_creation;
mod native_methods_impl;
pub mod native_methods_registry;
pub mod object;
pub mod stack_trace_element;
mod time;
pub mod value;
mod value_stack;
pub mod vm;
pub mod vm_error;
