# pykvd

Python bindings for the [KVD](https://github.com/key-value-document/kvd-spec)
key-value document format, built on [`kvd-rs`](https://github.com/key-value-document/kvd-rs)
via [PyO3](https://pyo3.rs) and [maturin](https://www.maturin.rs).

## Install

```sh
pip install pykvd
```

## Usage with files

`loads` and `dumps` work with strings, so pair them with `open()` for files.
This is the recommended way to read and write `.kvd` files.

```python
import pykvd

# Write Python objects to a KVD file (canonical form).
doc = {"app": {"port": 8080, "tags": ["web", "api"]}}
with open("config.kvd", "w") as f:
    f.write(pykvd.dumps(doc))

# Write the matching schema file.
with open("schema.kvd", "w") as f:
    f.write("app:\n  port: int\n  tags:\n    type: list\n    element: str\n")

# Read a KVD file into plain Python objects.
with open("config.kvd") as f:
    doc = pykvd.loads(f.read())
assert doc == {"app": {"port": 8080, "tags": ["web", "api"]}}

# Update one value in a file.
with open("config.kvd") as f:
    text = f.read()
text = pykvd.set(text, "app.port", 9090)
with open("config.kvd", "w") as f:
    f.write(text)

# Normalize a file to canonical form.
with open("config.kvd") as f:
    text = pykvd.canonical(f.read())
with open("config.kvd", "w") as f:
    f.write(text)

# Verify a file against a schema file.
with open("config.kvd") as f:
    doc_text = f.read()
with open("schema.kvd") as f:
    schema_text = f.read()
pykvd.verify(doc_text, schema_text)
```

## Usage with strings

```python
import pykvd

# Parse KVD text into plain Python objects.
doc = pykvd.loads("app:\n  port: 8080\n")

# Serialize Python objects to canonical KVD text.
pykvd.dumps({"app": {"port": 8080}})

# Normalize/validate KVD text to its canonical form.
pykvd.canonical("app:\n  port: 8080")

# Verify a document against a schema document.
pykvd.verify("port: 8080\n", "port: int\n")

# Check a schema document is well-formed.
pykvd.validate_schema("port: int\n")

# Structural editing with spec 8.5 paths.
pykvd.get("app:\n  port: 8080\n", "app.port")
pykvd.set("app:\n  port: 8080\n", "app.port", 9090)
pykvd.remove("app:\n  port: 8080\n", "app.port")
pykvd.remove("a:\n  b:\n    c: 1\n", "a.b.c", recursive=True)
```

Type notes:

- `dumps` accepts dicts, lists, tuples (as lists), str, int, float, bool,
  and None. Integers must fit in u64; larger values raise `KvdError`.
- Int literals beyond u64 parse but come back from `loads` as strings.
- The document root must be a mapping (spec 4).

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
