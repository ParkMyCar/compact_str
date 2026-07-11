//! PyO3 integration for CompactString
//!
//! This module provides seamless conversion between `CompactString` and Python strings,
//! allowing `CompactString` to be used in PyO3-based Python extensions.

use pyo3::prelude::*;
use pyo3::types::PyString;

use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
impl<'a, 'py> FromPyObject<'a, 'py> for CompactString {
    type Error = PyErr;

    fn extract(obj: pyo3::Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let s: &str = obj.extract()?;
        Ok(CompactString::from(s))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
impl<'py> IntoPyObject<'py> for CompactString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = core::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self.as_str()))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
impl<'py> IntoPyObject<'py> for &CompactString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = core::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self.as_str()))
    }
}

// These tests initialize an embedded Python interpreter and call into libpython via FFI.
// Miri cannot execute foreign functions, so they're excluded under Miri (the impl above is
// still checked by Miri; only these FFI-driven tests are skipped).
#[cfg(all(test, not(miri)))]
mod tests {
    use pyo3::prelude::*;
    use pyo3::types::PyString;

    use crate::CompactString;

    fn with_py<F, R>(f: F) -> R
    where
        F: for<'py> FnOnce(Python<'py>) -> R,
    {
        Python::initialize();
        Python::attach(f)
    }

    #[test]
    fn test_compact_string_into_py() {
        with_py(|py| {
            let cs = CompactString::from("hello world");
            let py_str = cs.into_pyobject(py).unwrap();
            assert_eq!(py_str.to_str().unwrap(), "hello world");
        });
    }

    #[test]
    fn test_compact_string_ref_into_py() {
        with_py(|py| {
            let cs = CompactString::from("hello world");
            let py_str = (&cs).into_pyobject(py).unwrap();
            assert_eq!(py_str.to_str().unwrap(), "hello world");
            // Original is still valid
            assert_eq!(cs.as_str(), "hello world");
        });
    }

    #[test]
    fn test_compact_string_from_py() {
        with_py(|py| {
            let py_str = PyString::new(py, "hello from python");
            let cs: CompactString = py_str.extract().unwrap();
            assert_eq!(cs.as_str(), "hello from python");
        });
    }

    #[test]
    fn test_roundtrip() {
        with_py(|py| {
            let original = CompactString::from("roundtrip test 🦀");
            let py_str = original.clone().into_pyobject(py).unwrap();
            let recovered: CompactString = py_str.extract().unwrap();
            assert_eq!(original, recovered);
        });
    }

    #[test]
    fn test_empty_string() {
        with_py(|py| {
            let cs = CompactString::new("");
            let py_str = cs.into_pyobject(py).unwrap();
            assert_eq!(py_str.to_str().unwrap(), "");

            let py_empty = PyString::new(py, "");
            let cs_empty: CompactString = py_empty.extract().unwrap();
            assert!(cs_empty.is_empty());
        });
    }

    #[test]
    fn test_unicode_string() {
        with_py(|py| {
            let unicode_str = "Hello, 世界! 🎉 Привет мир";
            let cs = CompactString::from(unicode_str);
            let py_str = cs.into_pyobject(py).unwrap();
            assert_eq!(py_str.to_str().unwrap(), unicode_str);

            let recovered: CompactString = py_str.extract().unwrap();
            assert_eq!(recovered.as_str(), unicode_str);
        });
    }

    #[test]
    fn test_inline_string() {
        with_py(|py| {
            // Short string that should be stored inline (not on heap)
            let short = CompactString::from("hi");
            assert!(!short.is_heap_allocated());

            let py_str = short.clone().into_pyobject(py).unwrap();
            let recovered: CompactString = py_str.extract().unwrap();
            assert_eq!(short, recovered);
        });
    }

    #[test]
    fn test_heap_string() {
        with_py(|py| {
            // Long string that should be stored on the heap
            let long = CompactString::from("this is a longer string that won't fit inline");
            assert!(long.is_heap_allocated());

            let py_str = long.clone().into_pyobject(py).unwrap();
            let recovered: CompactString = py_str.extract().unwrap();
            assert_eq!(long, recovered);
        });
    }
}
