use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::wrap_pyfunction;
use std::error::Error;

#[derive(Debug)]
struct MyStruct {
    field1: i32,
    field2: f32,
}

fn vec_to_dataframe(py: Python, data: Vec<MyStruct>) -> PyResult<&PyAny> {
    let pd = py.import("pandas")?.to_object(py);
    let list: &PyList = PyList::empty(py);
    for item in data {
        let dict: &PyDict = PyDict::new(py);
        dict.set_item("field1", item.field1)?;
        dict.set_item("field2", item.field2)?;
        list.append(dict)?;
    }
    pd.get("DataFrame")?.call1((list,))
}

#[pyfunction]
fn foo(df: &PyAny) -> PyResult<&PyAny> {
    let pd = Python::acquire_gil().python().import("pandas")?;
    let new_df = df.call_method0("copy")?;
    let cols = new_df.getattr("columns")?;
    cols.call_method1("sort", ("desc",))?;
    Ok(new_df)
}

#[pyfunction]
fn run(df: &PyAny) -> PyResult<&PyAny> {
    foo(df)
}

#[pymodule]
fn utils(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(foo)?)?;
    Ok(())
}

#[pymodule]
fn app(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run)?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    pyo3::prepare_freethreaded_python();
    
    let py_foo = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/python_app/utils/foo.py"));
    let py_app = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/python_app/app.py"));

    let my_vec = vec![
            MyStruct { field1: 1, field2: 12.0 },
            MyStruct { field1: 3, field2: 24.0 },
            MyStruct { field1: 5, field2: 36.0 },
        ];
    
    let (result_df, df) = Python::with_gil(|py| -> PyResult<_> {
        let df = vec_to_dataframe(py, my_vec)?;

        PyModule::from_code(py, py_foo, "utils.foo", "utils.foo")?;
        let app = PyModule::from_code(py, py_app, "", "")?
            .get("run")?
            .call1((df,))?;

        let result_df = app.extract(py)?;
        let new_df = result_df.call_method0("copy")?;
        let py_dataframe = new_df.to_object(py);

        Ok((result_df, py_dataframe))
    })?;

    println!("{:?}", result_df);
    println!("{:?}", df);
    
    Ok(())
}
