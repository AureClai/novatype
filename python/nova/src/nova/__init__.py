"""
Nova - Python figure generation for NovaType/Typst documents.

This package provides the @nova.figure decorator for creating figures
that can be seamlessly embedded in Typst documents.

Example:
    ```python
    import nova
    import matplotlib.pyplot as plt
    import numpy as np

    @nova.figure("sine-wave")
    def plot_sine():
        x = np.linspace(0, 2*np.pi, 100)
        plt.plot(x, np.sin(x))
        plt.title("Sine Wave")
    ```

    Then in your Typst document:
    ```typst
    #figure(
      nova("sine-wave"),
      caption: [A sine wave]
    )
    ```
"""

__version__ = "0.1.0"

from nova.decorators import figure
from nova.registry import FigureRegistry, get_registry
from nova.runner import generate_all, generate_figure

__all__ = [
    "__version__",
    "figure",
    "FigureRegistry",
    "get_registry",
    "generate_all",
    "generate_figure",
    "_execute_figure",
]


def _execute_figure(func, output_path: str) -> None:
    """
    Execute a figure function and save the output.

    This is called by nova-python (Rust) to execute individual figures.

    Args:
        func: The decorated figure function
        output_path: Path where the SVG should be saved
    """
    from nova.runner import _execute_and_save
    _execute_and_save(func, output_path)
