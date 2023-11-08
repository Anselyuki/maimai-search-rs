pub mod python_module {
    use pyo3::types::IntoPyDict;
    use pyo3::PyResult;
    use pyo3::Python;

    pub fn test_python() -> PyResult<()> {
        Python::with_gil(|py| {
            let sys = py.import("sys")?;
            let version: String = sys.getattr("version")?.extract()?;

            let locals = [("os", py.import("os")?)].into_py_dict(py);
            let code = "os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'";
            let user: String = py.eval(code, None, Some(&locals))?.extract()?;

            println!("Hello {}, I'm Python {}", user, version);
            Ok(())
        })
    }
}
