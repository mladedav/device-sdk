use pyo3::{
    prelude::*,
    types::{PyBytes, PyDict},
};

/// A Cloud-to-Device Message received from the Platform.
#[pyclass]
pub struct CloudToDeviceMessage {
    /// The binary content of the message.
    #[pyo3(get)]
    pub content: Py<PyBytes>,
    /// The additional message properties.
    #[pyo3(get)]
    pub properties: Py<PyDict>,
}
