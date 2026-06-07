//! # PyO3 Basics — Python-Rust Interop
//!
//! PyO3 allows writing Python extensions in Rust. This module covers the
//! patterns for exposing Rust code to Python, including classes, functions,
//! and error handling.
//!
//! ## PyO3 Key Attributes:
//!
//! - `#[pyclass]`: Mark a Rust struct as a Python class
//! - `#[pymethods]`: Define methods for a Python class
//! - `#[pyfunction]`: Define a standalone Python function
//! - `#[pymodule]`: Define a Python module
//!
//! ## Setup:
//!
//! ```toml
//! [dependencies]
//! pyo3 = { version = "0.20", features = ["extension-module"] }
//! ```
//!
//! ```rust,ignore
//! use pyo3::prelude::*;
//!
//! #[pyfunction]
//! fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//!     Ok((a + b).to_string())
//! }
//!
//! #[pymodule]
//! fn my_rust_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
//!     Ok(())
//! }
//! ```

/// Represents a Python-exposed data structure.
/// In real PyO3 code, this would have #[pyclass].
pub struct PyCompatibleStruct {
    pub name: String,
    pub value: f64,
    pub items: Vec<String>,
}

impl PyCompatibleStruct {
    pub fn new(name: &str, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
            items: Vec::new(),
        }
    }

    /// Method that would be exposed as __repr__ in Python.
    pub fn repr(&self) -> String {
        format!("PyCompatibleStruct(name='{}', value={})", self.name, self.value)
    }

    /// Method that would be exposed as a Python method.
    pub fn add_item(&mut self, item: &str) {
        self.items.push(item.to_string());
    }

    /// Property getter pattern.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Property setter pattern.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Convert to a Python-friendly dictionary representation.
    pub fn to_dict(&self) -> std::collections::HashMap<String, String> {
        let mut dict = std::collections::HashMap::new();
        dict.insert("name".into(), self.name.clone());
        dict.insert("value".into(), self.value.to_string());
        dict.insert("items_count".into(), self.items.len().to_string());
        dict
    }
}

/// Error type compatible with Python exceptions.
#[derive(Debug)]
pub enum PyCompatibleError {
    ValueError(String),
    TypeError(String),
    KeyError(String),
    RuntimeError(String),
}

impl std::fmt::Display for PyCompatibleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValueError(msg) => write!(f, "ValueError: {}", msg),
            Self::TypeError(msg) => write!(f, "TypeError: {}", msg),
            Self::KeyError(msg) => write!(f, "KeyError: {}", msg),
            Self::RuntimeError(msg) => write!(f, "RuntimeError: {}", msg),
        }
    }
}

impl std::error::Error for PyCompatibleError {}

/// Converter between Rust and Python types.
pub struct TypeConverter;

impl TypeConverter {
    /// Convert Rust string to Python-compatible representation.
    pub fn string_to_py(s: &str) -> String {
        s.to_string()
    }

    /// Convert Python string representation to Rust.
    pub fn py_to_string(py_val: &str) -> Result<String, PyCompatibleError> {
        Ok(py_val.to_string())
    }

    /// Convert Rust Vec to Python list representation.
    pub fn vec_to_py_list<T: std::fmt::Display>(items: &[T]) -> Vec<String> {
        items.iter().map(|item| format!("{}", item)).collect()
    }

    /// Convert Python list representation to Rust Vec.
    pub fn py_list_to_vec<T: std::str::FromStr>(items: &[String]) -> Result<Vec<T>, PyCompatibleError>
    where
        T::Err: std::fmt::Display,
    {
        items
            .iter()
            .map(|s| {
                s.parse::<T>().map_err(|e| {
                    PyCompatibleError::TypeError(format!("Cannot convert '{}': {}", s, e))
                })
            })
            .collect()
    }

    /// Convert Rust HashMap to Python dict representation.
    pub fn hashmap_to_py<K: std::fmt::Display, V: std::fmt::Display>(
        map: &std::collections::HashMap<K, V>,
    ) -> std::collections::HashMap<String, String> {
        map.iter()
            .map(|(k, v)| (format!("{}", k), format!("{}", v)))
            .collect()
    }
}

/// Trait for types that can be converted to/from Python.
pub trait PyConvertible: Sized {
    type Error;

    fn to_py_string(&self) -> String;
    fn from_py_string(s: &str) -> Result<Self, Self::Error>;
}

impl PyConvertible for i32 {
    type Error = PyCompatibleError;

    fn to_py_string(&self) -> String {
        self.to_string()
    }

    fn from_py_string(s: &str) -> Result<Self, Self::Error> {
        s.parse().map_err(|_| {
            PyCompatibleError::TypeError(format!("Cannot convert '{}' to i32", s))
        })
    }
}

impl PyConvertible for f64 {
    type Error = PyCompatibleError;

    fn to_py_string(&self) -> String {
        self.to_string()
    }

    fn from_py_string(s: &str) -> Result<Self, Self::Error> {
        s.parse().map_err(|_| {
            PyCompatibleError::TypeError(format!("Cannot convert '{}' to f64", s))
        })
    }
}

impl PyConvertible for String {
    type Error = PyCompatibleError;

    fn to_py_string(&self) -> String {
        self.clone()
    }

    fn from_py_string(s: &str) -> Result<Self, Self::Error> {
        Ok(s.to_string())
    }
}

/// Python-compatible iterator wrapper.
pub struct PyIterator<T> {
    items: Vec<T>,
    index: usize,
}

impl<T: Clone> PyIterator<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self { items, index: 0 }
    }

    /// Python's __next__ protocol.
    pub fn next(&mut self) -> Option<T> {
        if self.index < self.items.len() {
            let item = self.items[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    /// Python's __len__ protocol.
    pub fn len(&self) -> usize {
        self.items.len() - self.index
    }

    pub fn is_empty(&self) -> bool {
        self.index >= self.items.len()
    }

    /// Reset the iterator (Python's __iter__ returns self).
    pub fn reset(&mut self) {
        self.index = 0;
    }
}

/// Python context manager pattern (with statement).
pub struct PyContextManager<T> {
    resource: Option<T>,
    on_exit: Option<Box<dyn FnOnce(&mut T)>>,
}

impl<T> PyContextManager<T> {
    pub fn new<F>(resource: T, on_exit: F) -> Self
    where
        F: FnOnce(&mut T) + 'static,
    {
        Self {
            resource: Some(resource),
            on_exit: Some(Box::new(on_exit)),
        }
    }

    /// __enter__: Return the resource.
    pub fn enter(&mut self) -> Option<&mut T> {
        self.resource.as_mut()
    }

    /// __exit__: Cleanup and return whether to suppress exceptions.
    pub fn exit(&mut self) -> bool {
        if let (Some(mut resource), Some(on_exit)) = (self.resource.take(), self.on_exit.take()) {
            on_exit(&mut resource);
        }
        false // Don't suppress exceptions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_compatible_struct() {
        let mut obj = PyCompatibleStruct::new("test", 42.0);
        assert_eq!(obj.get_name(), "test");
        obj.add_item("item1");
        obj.add_item("item2");

        let dict = obj.to_dict();
        assert_eq!(dict.get("name").unwrap(), "test");
        assert_eq!(dict.get("items_count").unwrap(), "2");
    }

    #[test]
    fn test_py_compatible_struct_repr() {
        let obj = PyCompatibleStruct::new("hello", 3.14);
        let repr = obj.repr();
        assert!(repr.contains("hello"));
        assert!(repr.contains("3.14"));
    }

    #[test]
    fn test_py_compatible_error() {
        let err = PyCompatibleError::ValueError("bad value".into());
        assert!(format!("{}", err).contains("ValueError"));
    }

    #[test]
    fn test_type_converter_vec() {
        let items = vec![1, 2, 3];
        let py_list = TypeConverter::vec_to_py_list(&items);
        assert_eq!(py_list, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_type_converter_py_list() {
        let py_list = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let rust_vec: Vec<i32> = TypeConverter::py_list_to_vec(&py_list).unwrap();
        assert_eq!(rust_vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_type_converter_py_list_error() {
        let py_list = vec!["1".to_string(), "not_a_number".to_string()];
        let result: Result<Vec<i32>, _> = TypeConverter::py_list_to_vec(&py_list);
        assert!(result.is_err());
    }

    #[test]
    fn test_py_convertible_i32() {
        assert_eq!(i32::to_py_string(&42), "42");
        assert_eq!(i32::from_py_string("42").unwrap(), 42);
        assert!(i32::from_py_string("not_a_number").is_err());
    }

    #[test]
    fn test_py_convertible_f64() {
        assert_eq!(f64::to_py_string(&3.14), "3.14");
        assert!((f64::from_py_string("3.14").unwrap() - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_py_iterator() {
        let mut iter = PyIterator::new(vec![1, 2, 3]);
        assert_eq!(iter.len(), 3);

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.len(), 1);

        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
        assert!(iter.is_empty());
    }

    #[test]
    fn test_py_iterator_reset() {
        let mut iter = PyIterator::new(vec![10, 20]);
        iter.next();
        iter.reset();
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(10));
    }

    #[test]
    fn test_py_context_manager() {
        let mut ctx = PyContextManager::new(String::from("resource"), |r| {
            r.push_str("_cleaned");
        });

        {
            let resource = ctx.enter().unwrap();
            assert_eq!(resource, "resource");
            resource.push_str("_modified");
        }

        ctx.exit();
        // Cleanup was called
    }

    #[test]
    fn test_type_converter_hashmap() {
        let mut map = std::collections::HashMap::new();
        map.insert("key1", 100);
        map.insert("key2", 200);

        let py_dict = TypeConverter::hashmap_to_py(&map);
        assert_eq!(py_dict.get("key1").unwrap(), "100");
        assert_eq!(py_dict.get("key2").unwrap(), "200");
    }

    #[test]
    fn test_py_compatible_struct_set_name() {
        let mut obj = PyCompatibleStruct::new("old", 0.0);
        obj.set_name("new");
        assert_eq!(obj.get_name(), "new");
    }
}
