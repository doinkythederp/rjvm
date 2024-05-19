use alloc::string::String;

use snafu::Snafu;

use crate::value_stack::ValueStackError;

/// Various errors that are thrown when executing java bytecode
// TODO: this implementation is quite poor: we do not keep track of the origin
//  of the errors, and we do not keep many details
#[derive(Debug, Snafu, PartialEq, Eq)]
#[snafu(visibility(pub(crate)))]
pub enum VmError {
    #[snafu(display("unexpected error loading class: {message}"))]
    ClassLoadingError { message: String },

    /// TODO: this should become throwing a real `java.lang.NullPointerException`
    #[snafu(display("null pointer exception"))]
    NullPointerException,

    /// TODO: this should become throwing a real `java.lang.ClassNotFoundException`
    #[snafu(display("class not found: {class_name}"))]
    ClassNotFoundException { class_name: String },

    #[snafu(display("method not found: {class_name}.{method_name}#{method_type_descriptor}"))]
    MethodNotFoundException {
        class_name: String,
        method_name: String,
        method_type_descriptor: String,
    },

    #[snafu(display("field not found: {class_name}.{field_name}"))]
    FieldNotFoundException {
        class_name: String,
        field_name: String,
    },

    /// This is an overly generic error, abused to mean "something unexpected happened".
    /// It includes mostly errors that should be checked during the linking phase of the class file
    /// (which we have not implemented).
    #[snafu(display("validation exception - invalid class file"))]
    ValidationException,

    /// TODO: this should become throwing a real `java.lang.ArithmeticException`
    #[snafu(display("arithmetic exception"))]
    ArithmeticException,

    #[snafu(display("not yet implemented"))]
    NotImplemented,

    /// TODO: this should become throwing a real `java.lang.ArrayIndexOutOfBoundsException`
    #[snafu(display("array index out of bounds"))]
    ArrayIndexOutOfBoundsException,

    /// TODO: this should become throwing a real `java.lang.ClassCastException`
    #[snafu(display("class cast exception"))]
    ClassCastException,
}

// TODO: remove once we implement exceptions
impl From<ValueStackError> for VmError {
    fn from(_: ValueStackError) -> Self {
        Self::ValidationException
    }
}
