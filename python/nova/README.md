# Nova - Python Figures for Typst

Generate beautiful figures in Python for seamless inclusion in NovaType/Typst documents.

## Installation

```bash
pip install nova-typst
```

For additional visualization libraries:
```bash
pip install nova-typst[full]  # includes numpy, pandas, seaborn, plotly
```

## Quick Start

### 1. Create a figure in Python

```python
# figures/plots.py
import nova
import matplotlib.pyplot as plt
import numpy as np

@nova.figure("sine-wave")
def plot_sine():
    x = np.linspace(0, 2*np.pi, 100)
    plt.plot(x, np.sin(x), 'b-', linewidth=2)
    plt.xlabel("x")
    plt.ylabel("sin(x)")
    plt.title("Sine Wave")
    plt.grid(True, alpha=0.3)
```

### 2. Reference in your Typst document

```typst
#figure(
  nova("sine-wave"),
  caption: [A beautiful sine wave generated with Python]
) <fig:sine>
```

### 3. Compile with Nova

```bash
nova compile main.typ
```

Nova automatically:
- Discovers all `@nova.figure` decorated functions
- Executes only figures that have changed
- Caches SVG outputs for fast recompilation
- Injects figures into your Typst document

## Features

### File Dependencies

Declare file dependencies so figures regenerate when data changes:

```python
@nova.figure("results", depends=["data/results.csv"])
def plot_results():
    import pandas as pd
    df = pd.read_csv("data/results.csv")
    plt.bar(df["category"], df["value"])
```

### Multiple Figures Per File

Organize related figures together:

```python
# figures/analysis.py
import nova
import matplotlib.pyplot as plt

@nova.figure("histogram")
def plot_histogram():
    ...

@nova.figure("scatter")
def plot_scatter():
    ...

@nova.figure("boxplot")
def plot_boxplot():
    ...
```

### Reusable Code

Import and reuse functions across figures:

```python
# figures/utils.py
def apply_style():
    plt.style.use('seaborn-v0_8-whitegrid')
    plt.rcParams['font.family'] = 'serif'

# figures/plots.py
import nova
from utils import apply_style

@nova.figure("styled-plot")
def make_plot():
    apply_style()
    ...
```

## Configuration

Configure Python integration in `nova.toml`:

```toml
[python]
# Python executable (default: "python")
python = "python3"

# Directory containing figure scripts (default: "figures")
figures_dir = "figures"

# Cache directory (default: ".nova/cache")
cache_dir = ".nova/cache"

# Virtual environment (optional)
venv = ".venv"

# Execution timeout in seconds (default: 60)
timeout = 120

# Additional Python paths
python_path = ["src", "lib"]

# Environment variables
[python.env]
MPLBACKEND = "Agg"
```

## Best Practices

1. **Don't call `plt.savefig()`** - Nova handles saving automatically
2. **Don't call `plt.show()`** - This blocks execution
3. **Use `plt.figure()`** for new figures if creating multiple plots
4. **Return the figure** if you want explicit control: `return plt.gcf()`

## Project Structure

```
my-article/
├── main.typ              # Your document
├── nova.toml             # Configuration
├── figures/
│   ├── __init__.py
│   ├── analysis.py       # @nova.figure("results")
│   └── diagrams.py       # @nova.figure("architecture")
├── data/
│   └── measurements.csv
└── .nova/
    └── cache/            # Generated SVGs (git-ignored)
```

## License

MIT License
