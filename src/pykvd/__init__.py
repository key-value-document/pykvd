"""Python bindings for the KVD key-value document format.

This is a scaffold. It currently exposes a single function::

    import pykvd

    # Normalize/validate a KVD document to its canonical form.
    pykvd.canonical('app:\\n  port: 8080\\n')  # -> 'app:\\n  port: 8080\\n'

More of the kvd-rs API (schema verification, operations, serde-style
round-tripping) will be bound here as the package grows.
"""

from pykvd.pykvd import canonical

__all__ = ["canonical"]
