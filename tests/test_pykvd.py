import pytest

import pykvd


def test_loads_scalars():
    assert pykvd.loads("a: 1\nb: 1.5\nc: true\nd: null\ne: \"x\"\n") == {
        "a": 1,
        "b": 1.5,
        "c": True,
        "d": None,
        "e": "x",
    }


def test_loads_nested_and_lists():
    text = "app:\n  port: 8080\n  tags:\n    - \"web\"\n    - \"api\"\n"
    assert pykvd.loads(text) == {"app": {"port": 8080, "tags": ["web", "api"]}}


def test_loads_error():
    with pytest.raises(pykvd.KvdError):
        pykvd.loads("a:1\n")


def test_dumps_roundtrip():
    doc = {"app": {"port": 8080, "tags": ["web"]}}
    assert pykvd.loads(pykvd.dumps(doc)) == doc


def test_dumps_dict_keys():
    doc = {"m": {"a.b/c": 1}}
    out = pykvd.dumps(doc)
    assert "= \"a.b/c\"" in out
    assert pykvd.loads(out) == doc


def test_canonical():
    assert pykvd.canonical("app:\n  port: 8080\n") == "app:\n  port: 8080\n"


def test_verify_ok_and_violation():
    pykvd.verify("port: 8080\n", "port: int\n")
    with pytest.raises(pykvd.SchemaError):
        pykvd.verify("port: \"x\"\n", "port: int\n")


def test_verify_malformed_schema():
    with pytest.raises(pykvd.SchemaError):
        pykvd.verify("a: 1\n", "a: 5\n")


def test_validate_schema():
    pykvd.validate_schema("port: int\n")
    with pytest.raises(pykvd.SchemaError):
        pykvd.validate_schema("a: 5\n")


def test_get_set_remove():
    text = "app:\n  port: 8080\n"
    assert pykvd.get(text, "app.port") == 8080
    with pytest.raises(KeyError):
        pykvd.get(text, "app.missing")
    updated = pykvd.set(text, "app.port", 9090)
    assert pykvd.loads(updated) == {"app": {"port": 9090}}
    pruned = pykvd.remove(text, "app.port")
    assert pykvd.loads(pruned) == {"app": {}}


def test_get_bad_path():
    with pytest.raises(pykvd.OpError):
        pykvd.get("a: 1\n", "")


def test_error_hierarchy():
    assert issubclass(pykvd.SchemaError, pykvd.KvdError)
    assert issubclass(pykvd.OpError, pykvd.KvdError)
    assert issubclass(pykvd.KvdError, ValueError)
