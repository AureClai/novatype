"""Tests for the @nova.figure decorator."""

import pytest
from pathlib import Path

from nova.decorators import figure
from nova.registry import get_registry, reset_registry


@pytest.fixture(autouse=True)
def clean_registry():
    """Reset the registry before each test."""
    reset_registry()
    yield
    reset_registry()


def test_figure_decorator_registers_function():
    """Test that @figure registers the function in the registry."""

    @figure("test-fig")
    def make_figure():
        pass

    registry = get_registry()
    assert "test-fig" in registry
    info = registry["test-fig"]
    assert info.name == "test-fig"
    assert info.func is make_figure


def test_figure_decorator_with_dependencies():
    """Test that dependencies are captured."""

    @figure("with-deps", depends=["data.csv", "config.json"])
    def make_figure():
        pass

    registry = get_registry()
    info = registry["with-deps"]
    assert len(info.dependencies) == 2
    assert Path("data.csv") in info.dependencies
    assert Path("config.json") in info.dependencies


def test_figure_decorator_preserves_function():
    """Test that the decorator preserves function behavior."""

    @figure("preserved")
    def add_numbers(a, b):
        return a + b

    # Function should still work
    assert add_numbers(2, 3) == 5


def test_figure_decorator_stores_metadata():
    """Test that metadata is stored on the function."""

    @figure("with-meta", format="png", dpi=300)
    def make_figure():
        pass

    assert make_figure._nova_figure is True
    assert make_figure._nova_name == "with-meta"
    assert make_figure._nova_format == "png"
    assert make_figure._nova_dpi == 300


def test_duplicate_figure_name_raises():
    """Test that registering duplicate names raises an error."""

    @figure("duplicate")
    def first():
        pass

    with pytest.raises(ValueError, match="already registered"):
        @figure("duplicate")
        def second():
            pass


def test_figure_defaults():
    """Test default values for optional parameters."""

    @figure("defaults")
    def make_figure():
        pass

    registry = get_registry()
    info = registry["defaults"]
    assert info.format == "svg"
    assert info.dpi == 150
    assert info.dependencies == []
