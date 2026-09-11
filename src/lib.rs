use kvd_rs::deserialize::from_str;
use kvd_rs::ops::{self, Path};
use kvd_rs::schema::{self, VerifyError, Violation};
use kvd_rs::serialize::to_string as serialize_to_string;
use kvd_rs::value::{Node, Scalar, Shape};
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyDict, PyFloat, PyList, PyNone, PyString};

pyo3::create_exception!(pykvd, KvdError, PyValueError);
pyo3::create_exception!(pykvd, SchemaError, KvdError);
pyo3::create_exception!(pykvd, OpError, KvdError);

fn parse_err(e: kvd_rs::error::Error) -> PyErr {
    KvdError::new_err(format!(
        "{}:{}: {}: {}",
        e.line,
        e.col,
        e.kind.as_str(),
        e.message
    ))
}

fn serialize_err(e: kvd_rs::serialize::SerializeError) -> PyErr {
    KvdError::new_err(e.to_string())
}

fn op_err(e: kvd_rs::ops::OpError) -> PyErr {
    OpError::new_err(e.to_string())
}

fn verify_err(e: VerifyError) -> PyErr {
    match e {
        VerifyError::ParseDoc(e) => parse_err(e),
        VerifyError::ParseSchema(e) => SchemaError::new_err(format!("schema parse error: {e}")),
        VerifyError::Violations(vs) => SchemaError::new_err(violations_text(&vs)),
        VerifyError::SchemaMalformed(vs) => {
            SchemaError::new_err(format!("malformed schema:\n{}", violations_text(&vs)))
        }
        _ => SchemaError::new_err(e.to_string()),
    }
}

fn violations_text(vs: &[Violation]) -> String {
    vs.iter()
        .map(|v| {
            if v.path.is_empty() {
                v.message.clone()
            } else {
                format!("{}: {}", v.path, v.message)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn node_to_py<'py>(py: Python<'py>, node: &Node) -> PyResult<Bound<'py, PyAny>> {
    match node {
        Node::Scalar(s) => scalar_to_py(py, s),
        Node::Map(m) | Node::Dict(m) => {
            let d = PyDict::new(py);
            for (k, v) in m.iter() {
                d.set_item(k, node_to_py(py, v)?)?;
            }
            Ok(d.into_any())
        }
        Node::List(items) => {
            let out = PyList::empty(py);
            for item in items {
                out.append(node_to_py(py, item)?)?;
            }
            Ok(out.into_any())
        }
        _ => Err(KvdError::new_err("unsupported node kind")),
    }
}

fn scalar_to_py<'py>(py: Python<'py>, s: &Scalar) -> PyResult<Bound<'py, PyAny>> {
    match s.shape {
        Shape::Int => {
            let digits: String = s.text.chars().filter(|c| *c != '_').collect();
            if let Ok(i) = digits.parse::<i64>() {
                return Ok(i.into_pyobject(py)?.into_any());
            }
            if let Ok(u) = digits.parse::<u64>() {
                return Ok(u.into_pyobject(py)?.into_any());
            }
            Ok(PyString::new(py, &s.text).into_any())
        }
        Shape::Float => match s.text.parse::<f64>() {
            Ok(f) if f.is_finite() => Ok(PyFloat::new(py, f).into_any()),
            _ => Ok(PyString::new(py, &s.text).into_any()),
        },
        Shape::Bool => Ok(PyBool::new(py, s.text == "true").to_owned().into_any()),
        Shape::Null => Ok(PyNone::get(py).to_owned().into_any()),
        Shape::Str => Ok(PyString::new(py, &s.text).into_any()),
    }
}

fn py_to_node(value: &Bound<'_, PyAny>) -> PyResult<Node> {
    if value.is_none() {
        return Ok(Node::scalar(Shape::Null, "null"));
    }
    if let Ok(b) = value.cast::<PyBool>() {
        return Ok(Node::scalar(Shape::Bool, b.is_true().to_string()));
    }
    if let Ok(i) = value.extract::<i64>() {
        return Ok(Node::scalar(Shape::Int, i.to_string()));
    }
    if let Ok(f) = value.extract::<f64>() {
        if !f.is_finite() {
            return Err(KvdError::new_err(
                "non-finite floats have no KVD representation",
            ));
        }
        let text = if f.fract() == 0.0 && f.abs() < 1e15 {
            format!("{f:.1}")
        } else {
            format!("{f}")
        };
        return Ok(Node::scalar(Shape::Float, text));
    }
    if let Ok(s) = value.cast::<PyString>() {
        return Ok(Node::scalar(Shape::Str, s.to_string_lossy().into_owned()));
    }
    if let Ok(d) = value.cast::<PyDict>() {
        let mut m = kvd_rs::value::Map::new();
        let mut dict = kvd_rs::value::Map::new();
        for (k, v) in d.iter() {
            let key: String = k
                .extract()
                .map_err(|_| KvdError::new_err("mapping keys must be strings"))?;
            let node = py_to_node(&v)?;
            if kvd_rs::grammar::is_key(&key) {
                m.insert(key, node);
            } else {
                dict.insert(key, node);
            }
        }
        if !m.is_empty() && !dict.is_empty() {
            return Err(KvdError::new_err(
                "mapping mixes bare keys and opaque dict keys; split into separate mappings",
            ));
        }
        if !dict.is_empty() {
            return Ok(Node::dict(dict));
        }
        return Ok(Node::map(m));
    }
    if let Ok(l) = value.cast::<PyList>() {
        let mut items = Vec::with_capacity(l.len());
        for item in l.iter() {
            items.push(py_to_node(&item)?);
        }
        return Ok(Node::list(items));
    }
    Err(KvdError::new_err(format!(
        "unsupported Python type: {}",
        value
            .get_type()
            .name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "?".to_string())
    )))
}

/// Parse a KVD document into Python objects (dicts, lists, str, int, float, bool, None).
///
/// Raises `KvdError` if the input is not valid KVD.
#[pyfunction]
fn loads(py: Python<'_>, text: &str) -> PyResult<Py<PyAny>> {
    let doc = from_str(text).map_err(parse_err)?;
    Ok(node_to_py(py, &doc)?.into())
}

/// Serialize Python objects (dicts, lists, str, int, float, bool, None) to canonical KVD.
///
/// Raises `KvdError` on unsupported types or non-finite floats.
#[pyfunction]
fn dumps(_py: Python<'_>, value: Bound<'_, PyAny>) -> PyResult<String> {
    let node = py_to_node(&value)?;
    serialize_to_string(&node).map_err(serialize_err)
}

/// Parse a KVD document and return its canonical serialization.
///
/// Raises `KvdError` if the input is not valid KVD.
#[pyfunction]
fn canonical(text: &str) -> PyResult<String> {
    let doc = from_str(text).map_err(parse_err)?;
    serialize_to_string(&doc).map_err(serialize_err)
}

/// Verify a KVD document against a KVD schema document.
///
/// Raises `SchemaError` listing violations, or on schema parse problems.
/// Raises `KvdError` if the document itself does not parse.
#[pyfunction]
fn verify(doc: &str, schema: &str) -> PyResult<()> {
    schema::verify_from_str(doc, schema).map_err(verify_err)
}

/// Check that a schema document is well-formed.
///
/// Raises `SchemaError` describing schema problems.
/// Raises `KvdError` if the schema text does not parse.
#[pyfunction]
fn validate_schema(schema: &str) -> PyResult<()> {
    let node = from_str(schema).map_err(parse_err)?;
    let mut out = Vec::new();
    schema::validate_schema(&node, "", &mut out);
    if out.is_empty() {
        Ok(())
    } else {
        Err(SchemaError::new_err(format!(
            "malformed schema:\n{}",
            violations_text(&out)
        )))
    }
}

/// Read the value at `path` (spec 8.5: `a.b[0].c`) from a KVD document.
///
/// Returns Python objects like `loads`. Raises `OpError` on bad paths,
/// `KeyError` on missing keys, `IndexError` on out-of-range indices.
#[pyfunction]
fn get(py: Python<'_>, text: &str, path: &str) -> PyResult<Py<PyAny>> {
    let doc = from_str(text).map_err(parse_err)?;
    let p = Path::parse(path).map_err(op_err)?;
    match ops::get(&doc, &p) {
        Some(node) => Ok(node_to_py(py, node)?.into()),
        None => Err(missing_err(&doc, &p)),
    }
}

fn missing_err(doc: &Node, path: &Path) -> PyErr {
    let mut cur = doc;
    for seg in path.segments() {
        match seg {
            kvd_rs::ops::Segment::Key(k) => match cur.as_keyed().and_then(|m| m.get(k)) {
                Some(next) => cur = next,
                None => return PyKeyError::new_err(k.clone()),
            },
            kvd_rs::ops::Segment::Index(i) => match cur.as_list().and_then(|l| l.get(*i)) {
                Some(next) => cur = next,
                None => {
                    return PyErr::new::<pyo3::exceptions::PyIndexError, _>(format!(
                        "index out of bounds: {i}"
                    ));
                }
            },
        }
    }
    OpError::new_err("path did not resolve")
}

/// Set the value at `path` in a KVD document; returns the updated canonical text.
///
/// Intermediate maps are created as needed. List indices must already exist.
#[pyfunction]
fn set(text: &str, path: &str, value: Bound<'_, PyAny>) -> PyResult<String> {
    let mut doc = from_str(text).map_err(parse_err)?;
    let p = Path::parse(path).map_err(op_err)?;
    let node = py_to_node(&value)?;
    ops::set(&mut doc, &p, node).map_err(op_err)?;
    serialize_to_string(&doc).map_err(serialize_err)
}

/// Remove the value at `path` from a KVD document; returns the updated canonical text.
#[pyfunction]
fn remove(text: &str, path: &str) -> PyResult<String> {
    let mut doc = from_str(text).map_err(parse_err)?;
    let p = Path::parse(path).map_err(op_err)?;
    ops::remove(&mut doc, &p).map_err(op_err)?;
    serialize_to_string(&doc).map_err(serialize_err)
}

#[pymodule]
fn pykvd(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(loads, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    m.add_function(wrap_pyfunction!(canonical, m)?)?;
    m.add_function(wrap_pyfunction!(verify, m)?)?;
    m.add_function(wrap_pyfunction!(validate_schema, m)?)?;
    m.add_function(wrap_pyfunction!(get, m)?)?;
    m.add_function(wrap_pyfunction!(set, m)?)?;
    m.add_function(wrap_pyfunction!(remove, m)?)?;
    m.add("KvdError", m.py().get_type::<KvdError>())?;
    m.add("SchemaError", m.py().get_type::<SchemaError>())?;
    m.add("OpError", m.py().get_type::<OpError>())?;
    Ok(())
}
