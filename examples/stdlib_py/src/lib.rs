//! `wire_weaver`, the extension module of [`wire_weaver_py`] at the commit `Cargo.lock` pins, for the Python scripts
//! of the standard library examples and their tests (`gpio.py`, `tests/`). A reader installs the shipped wheel
//! instead (vhrdtech/ww_tools), which wraps the same `init`.

use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "wire_weaver")]
fn stdlib_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    wire_weaver_py::init(m)
}
