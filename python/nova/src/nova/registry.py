"""
Figure registry for tracking decorated figures.
"""

from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable, Dict, List, Optional

# Global registry instance
_registry: Optional["FigureRegistry"] = None


@dataclass
class FigureInfo:
    """Information about a registered figure."""

    name: str
    """Unique name of the figure."""

    func: Callable
    """The figure-generating function."""

    source_file: Path
    """Path to the source file containing the function."""

    dependencies: List[Path] = field(default_factory=list)
    """List of file dependencies."""

    format: str = "svg"
    """Output format (svg or png)."""

    dpi: int = 150
    """DPI for PNG output."""

    generated_path: Optional[Path] = None
    """Path to the generated figure (after generation)."""


class FigureRegistry:
    """
    Registry of all figures defined with @nova.figure.

    This is a singleton that collects all figure definitions when
    Python modules are imported.
    """

    def __init__(self):
        self._figures: Dict[str, FigureInfo] = {}

    def register(
        self,
        name: str,
        func: Callable,
        source_file: Path,
        dependencies: List[Path],
        format: str = "svg",
        dpi: int = 150,
    ) -> None:
        """
        Register a figure.

        Args:
            name: Unique figure name
            func: The figure function
            source_file: Path to source file
            dependencies: File dependencies
            format: Output format
            dpi: DPI for PNG

        Raises:
            ValueError: If a figure with this name already exists
        """
        if name in self._figures:
            existing = self._figures[name]
            raise ValueError(
                f"Figure '{name}' already registered in {existing.source_file}. "
                f"Cannot re-register in {source_file}."
            )

        self._figures[name] = FigureInfo(
            name=name,
            func=func,
            source_file=source_file,
            dependencies=dependencies,
            format=format,
            dpi=dpi,
        )

    def get(self, name: str) -> Optional[FigureInfo]:
        """Get a figure by name."""
        return self._figures.get(name)

    def __getitem__(self, name: str) -> FigureInfo:
        """Get a figure by name, raising KeyError if not found."""
        if name not in self._figures:
            raise KeyError(f"Figure '{name}' not found in registry")
        return self._figures[name]

    def __contains__(self, name: str) -> bool:
        """Check if a figure is registered."""
        return name in self._figures

    def __len__(self) -> int:
        """Get the number of registered figures."""
        return len(self._figures)

    def __iter__(self):
        """Iterate over figure names."""
        return iter(self._figures)

    def items(self):
        """Iterate over (name, info) pairs."""
        return self._figures.items()

    def values(self):
        """Iterate over FigureInfo objects."""
        return self._figures.values()

    def names(self) -> List[str]:
        """Get all registered figure names."""
        return list(self._figures.keys())

    def clear(self) -> None:
        """Clear all registered figures."""
        self._figures.clear()


def get_registry() -> FigureRegistry:
    """Get the global figure registry."""
    global _registry
    if _registry is None:
        _registry = FigureRegistry()
    return _registry


def reset_registry() -> None:
    """Reset the global registry (mainly for testing)."""
    global _registry
    _registry = None
