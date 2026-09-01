# pykvd

Python bindings for the [KVD](https://github.com/key-value-document/kvd-spec)
key-value document format, built on [`kvd-rs`](https://github.com/key-value-document/kvd-rs)
via [PyO3](https://pyo3.rs) and [maturin](https://www.maturin.rs).

> Status: scaffold. Currently exposes a minimal, validated surface.

## Install (from a built wheel)

```sh
pip install pykvd
```

## Usage

```python
import pykvd

# Normalize/validate a KVD document to its canonical form.
pykvd.canonical("app:\n  port: 8080")
```

A `ValueError` is raised when the input is not valid KVD.

## Development

```sh
pip install maturin
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop
python -c "import pykvd; print(pykvd.canonical('a: 1\n'))"
```

## License

MIT
