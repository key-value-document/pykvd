"""Python bindings for the KVD key-value document format.

Built on kvd-rs via PyO3. Documents map to plain Python objects:
mappings become dicts, lists become lists, and scalars become
str, int, float, bool, or None (for `null`).

Example::

    import pykvd

    doc = pykvd.loads('app:\\n  port: 8080\\n')
    pykvd.dumps(doc)  # canonical KVD text
    pykvd.verify('port: 8080\\n', 'port: int\\n')
    pykvd.get('app:\\n  port: 8080\\n', 'app.port')
"""

from pykvd.pykvd import (
    KvdError,
    OpError,
    SchemaError,
    canonical,
    dumps,
    get,
    loads,
    remove,
    set,
    validate_schema,
    verify,
)

__all__ = [
    "KvdError",
    "OpError",
    "SchemaError",
    "canonical",
    "dumps",
    "get",
    "loads",
    "remove",
    "set",
    "validate_schema",
    "verify",
]
