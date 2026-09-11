# pykvd

Python bindings for the [KVD](https://github.com/key-value-document/kvd-spec)
key-value document format, built on [`kvd-rs`](https://github.com/key-value-document/kvd-rs)
via [PyO3](https://pyo3.rs) and [maturin](https://www.maturin.rs).

## Install (from a built wheel)

```sh
pip install pykvd
```

## Usage

```python
import pykvd

# Parse KVD into plain Python objects.
doc = pykvd.loads("app:\n  port: 8080\n")
assert doc == {"app": {"port": 8080}}

# Serialize Python objects to canonical KVD.
pykvd.dumps({"app": {"port": 8080}})

# Normalize/validate a KVD document to its canonical form.
pykvd.canonical("app:\n  port: 8080")

# Verify a document against a schema document.
pykvd.verify("port: 8080\n", "port: int\n")

# Check a schema document is well-formed.
pykvd.validate_schema("port: int\n")

# Structural editing with spec 8.5 paths.
pykvd.get("app:\n  port: 8080\n", "app.port")
pykvd.set("app:\n  port: 8080\n", "app.port", 9090)
pykvd.remove("app:\n  port: 8080\n", "app.port")
```

Errors:

- `KvdError` (a `ValueError`) for parse and serialization failures.
- `SchemaError` (a `KvdError`) for verification failures and malformed schemas.
- `OpError` (a `KvdError`) for bad operation paths. Missing keys raise
  `KeyError` and out-of-range indices raise `IndexError`.

## Development

```sh
pip install maturin pytest
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop
pytest tests/
```

## License

MIT
