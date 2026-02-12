"""
Figure execution and generation.
"""

import importlib
import importlib.util
import sys
from pathlib import Path
from typing import List, Optional, Tuple

import matplotlib
import matplotlib.pyplot as plt

from nova.registry import FigureInfo, get_registry


def generate_figure(name: str, output_dir: Path) -> Path:
    """
    Generate a single figure by name.

    Args:
        name: The figure name (from @nova.figure decorator)
        output_dir: Directory to save the generated figure

    Returns:
        Path to the generated SVG/PNG file

    Raises:
        KeyError: If the figure is not found in the registry
    """
    registry = get_registry()
    info = registry[name]

    output_path = output_dir / f"{name}.{info.format}"
    _execute_and_save(info.func, str(output_path), info.format, info.dpi)

    info.generated_path = output_path
    return output_path


def generate_all(
    figures_dir: str,
    output_dir: str,
) -> List[Tuple[str, str]]:
    """
    Discover and generate all figures in a directory.

    This function:
    1. Imports all Python files in figures_dir
    2. Collects @nova.figure decorated functions
    3. Generates each figure to output_dir
    4. Prints name:path pairs for each generated figure

    Args:
        figures_dir: Directory containing Python figure files
        output_dir: Directory to save generated figures

    Returns:
        List of (name, path) tuples for generated figures
    """
    figures_path = Path(figures_dir)
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    # Import all Python files to register figures
    _import_figures_directory(figures_path)

    # Generate all registered figures
    registry = get_registry()
    results = []

    for name, info in registry.items():
        try:
            path = generate_figure(name, output_path)
            results.append((name, str(path)))
            # Print for nova-python to parse
            print(f"{name}:{path}")
        except Exception as e:
            print(f"ERROR:{name}:{e}", file=sys.stderr)

    return results


def _execute_and_save(
    func,
    output_path: str,
    format: str = "svg",
    dpi: int = 150,
) -> None:
    """
    Execute a figure function and save the output.

    Args:
        func: The figure function to execute
        output_path: Path to save the figure
        format: Output format (svg/png)
        dpi: DPI for PNG output
    """
    # Use non-interactive backend
    matplotlib.use("Agg")

    # Clear any existing figures
    plt.close("all")

    # Execute the function
    result = func()

    # Get the current figure
    if result is not None and hasattr(result, "savefig"):
        # Function returned a figure object
        fig = result
    else:
        # Get current figure from pyplot
        fig = plt.gcf()

    # Save the figure
    save_kwargs = {
        "format": format,
        "bbox_inches": "tight",
        "transparent": True,
    }

    if format == "png":
        save_kwargs["dpi"] = dpi

    fig.savefig(output_path, **save_kwargs)
    plt.close(fig)


def _import_figures_directory(figures_dir: Path) -> None:
    """
    Import all Python files in a directory to register figures.

    Args:
        figures_dir: Directory containing Python figure files
    """
    if not figures_dir.exists():
        return

    # Add figures directory to path
    figures_dir_str = str(figures_dir.resolve())
    if figures_dir_str not in sys.path:
        sys.path.insert(0, figures_dir_str)

    # Also add parent for package imports
    parent_str = str(figures_dir.parent.resolve())
    if parent_str not in sys.path:
        sys.path.insert(0, parent_str)

    # Import all .py files
    for py_file in figures_dir.rglob("*.py"):
        if py_file.name.startswith("_"):
            continue

        try:
            _import_module_from_path(py_file)
        except Exception as e:
            print(f"Warning: Failed to import {py_file}: {e}", file=sys.stderr)


def _import_module_from_path(path: Path) -> None:
    """Import a Python module from its file path."""
    module_name = path.stem

    # Create a unique module name to avoid conflicts
    unique_name = f"nova_figures.{path.parent.name}.{module_name}"

    spec = importlib.util.spec_from_file_location(unique_name, path)
    if spec is None or spec.loader is None:
        raise ImportError(f"Cannot load module from {path}")

    module = importlib.util.module_from_spec(spec)
    sys.modules[unique_name] = module
    spec.loader.exec_module(module)
