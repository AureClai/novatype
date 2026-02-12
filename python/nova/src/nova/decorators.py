"""
Decorators for defining Nova figures.
"""

from functools import wraps
from pathlib import Path
from typing import Callable, List, Optional, Union

from nova.registry import get_registry


def figure(
    name: str,
    *,
    depends: Optional[List[Union[str, Path]]] = None,
    format: str = "svg",
    dpi: int = 150,
) -> Callable:
    """
    Decorator to mark a function as a Nova figure.

    The decorated function should create a matplotlib figure. Nova will
    automatically capture and save it as SVG for inclusion in Typst documents.

    Args:
        name: Unique identifier for the figure. Used in Typst as `nova("name")`.
        depends: List of file paths this figure depends on. If any dependency
                 changes, the figure will be regenerated.
        format: Output format ("svg" or "png"). SVG is recommended for quality.
        dpi: Resolution for PNG output (ignored for SVG).

    Returns:
        Decorator function.

    Example:
        ```python
        @nova.figure("results", depends=["data/results.csv"])
        def plot_results():
            df = pd.read_csv("data/results.csv")
            plt.bar(df["x"], df["y"])
            plt.title("Results")
        ```

    Notes:
        - The function should NOT call plt.savefig() - Nova handles this.
        - The function should NOT call plt.show() - this blocks execution.
        - Multiple figures can be defined in a single file.
        - Use plt.figure() to create new figures if needed.
    """
    depends = depends or []
    depends = [Path(p) if isinstance(p, str) else p for p in depends]

    def decorator(func: Callable) -> Callable:
        # Register the figure
        registry = get_registry()
        registry.register(
            name=name,
            func=func,
            source_file=_get_source_file(func),
            dependencies=depends,
            format=format,
            dpi=dpi,
        )

        @wraps(func)
        def wrapper(*args, **kwargs):
            return func(*args, **kwargs)

        # Store metadata on the function
        wrapper._nova_figure = True
        wrapper._nova_name = name
        wrapper._nova_depends = depends
        wrapper._nova_format = format
        wrapper._nova_dpi = dpi

        return wrapper

    return decorator


def _get_source_file(func: Callable) -> Path:
    """Get the source file path of a function."""
    import inspect

    try:
        source_file = inspect.getfile(func)
        return Path(source_file).resolve()
    except (TypeError, OSError):
        return Path("<unknown>")
