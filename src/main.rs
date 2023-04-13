use std::error::Error;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

#[derive(Debug)]
struct MyStruct {
    field1: i32,
    field2: f64,
}

fn vec_to_dataframe(py: Python, my_vec_of_structs: Vec<MyStruct>) -> PyResult<PyObject> {
    let pandas = PyModule::import(py, "pandas")?;
    let dataframe_class = pandas.getattr("DataFrame")?;

    let mut rows = Vec::new();
    for item in my_vec_of_structs {
        let pydict = PyDict::new(py);
        pydict.set_item("field1", item.field1)?;
        pydict.set_item("field2", item.field2)?;
        rows.push(pydict);
    }

    let pylist = PyList::new(py, &rows);

    let py_dataframe = dataframe_class.call1((pylist,))?;

    Ok(py_dataframe.into_py(py))
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
    
    let df = Python::with_gil(|py| -> PyResult<_> {
        
        let df = vec_to_dataframe(py, my_vec)?;
        
        PyModule::from_code(py, py_foo, "utils.foo", "utils.foo")?;
        let app: Py<PyAny> = PyModule::from_code(py, py_app, "", "")?
            .getattr("run")?
            .extract::<&PyAny>()?
            .into();


        let df_arg = df.to_object(py);
        let result: Py<PyAny> = app.call1(py, (df_arg,))?;
        let result_df: &PyAny = result.extract(py)?;

        let new_df: &PyAny = result_df.call_method0("copy")?;
        let py_dataframe = new_df.to_object(py);

        Ok(py_dataframe)
    })?;

    //println!("from_python: {}", from_python);
    println!("df: {}", df);
    
    Ok(())
}
