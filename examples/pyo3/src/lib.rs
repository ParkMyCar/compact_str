//! Example PyO3 module demonstrating CompactString integration.
//!
//! This creates a Python extension module that uses `CompactString` internally
//! while exposing a Python-friendly interface.
//!
//! # Building
//!
//! ```bash
//! # Using maturin (recommended)
//! pip install maturin
//! cd examples/pyo3
//! maturin develop
//!
//! # Then in Python:
//! # >>> import compact_str_example
//! # >>> person = compact_str_example.Person("Alice", 30)
//! # >>> person.greet()
//! # 'Hello, my name is Alice and I am 30 years old!'
//! ```

use compact_str::CompactString;
use pyo3::prelude::*;

/// A simple Person struct that uses CompactString internally.
///
/// This demonstrates how CompactString can be used seamlessly with PyO3,
/// with automatic conversion to/from Python strings.
#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct Person {
    name: CompactString,
    age: u32,
}

#[pymethods]
impl Person {
    /// Create a new Person with the given name and age.
    #[new]
    fn new(name: CompactString, age: u32) -> Self {
        Person { name, age }
    }

    /// Get the person's name.
    #[getter]
    fn name(&self) -> CompactString {
        self.name.clone()
    }

    /// Set the person's name.
    #[setter]
    fn set_name(&mut self, name: CompactString) {
        self.name = name;
    }

    /// Get the person's age.
    #[getter]
    fn age(&self) -> u32 {
        self.age
    }

    /// Set the person's age.
    #[setter]
    fn set_age(&mut self, age: u32) {
        self.age = age;
    }

    /// Generate a greeting message.
    fn greet(&self) -> CompactString {
        CompactString::from(format!(
            "Hello, my name is {} and I am {} years old!",
            self.name, self.age
        ))
    }

    fn __repr__(&self) -> String {
        format!("Person(name='{}', age={})", self.name, self.age)
    }
}

/// Concatenate two strings and return the result as a CompactString.
///
/// This function demonstrates direct CompactString usage in function parameters
/// and return types.
#[pyfunction]
fn concat_strings(a: CompactString, b: CompactString) -> CompactString {
    let mut result = a;
    result.push_str(&b);
    result
}

/// Create a greeting message for the given name.
#[pyfunction]
fn create_greeting(name: CompactString) -> CompactString {
    CompactString::from(format!("Hello, {}!", name))
}

/// Check if a string would be stored inline (on the stack) by CompactString.
///
/// CompactString stores short strings inline without heap allocation.
/// This function helps demonstrate that optimization.
#[pyfunction]
fn would_be_inline(s: CompactString) -> bool {
    !s.is_heap_allocated()
}

/// The Python module definition.
#[pymodule]
fn compact_str_example(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Person>()?;
    m.add_function(wrap_pyfunction!(concat_strings, m)?)?;
    m.add_function(wrap_pyfunction!(create_greeting, m)?)?;
    m.add_function(wrap_pyfunction!(would_be_inline, m)?)?;
    Ok(())
}
