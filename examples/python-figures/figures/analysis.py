"""
Data analysis figures with file dependencies.

These figures demonstrate how to use the `depends` parameter
to track data file dependencies for cache invalidation.
"""

import nova
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path


@nova.figure("data-plot", depends=["data/measurements.csv"])
def plot_data():
    """Plot data from a CSV file."""
    # Read data (simplified - in real usage, use pandas)
    data_file = Path(__file__).parent.parent / "data" / "measurements.csv"

    if data_file.exists():
        # Parse CSV manually (for demo without pandas dependency)
        lines = data_file.read_text().strip().split("\n")
        headers = lines[0].split(",")
        data = []
        for line in lines[1:]:
            data.append([float(x) for x in line.split(",")])
        data = np.array(data)
        x = data[:, 0]
        y = data[:, 1]
    else:
        # Generate sample data if file doesn't exist
        x = np.linspace(0, 10, 20)
        y = 2 * x + np.random.normal(0, 1, 20)

    plt.figure(figsize=(8, 5))
    plt.scatter(x, y, s=80, alpha=0.7, edgecolors="black")
    plt.xlabel("X Measurement")
    plt.ylabel("Y Measurement")
    plt.title("Experimental Data")
    plt.grid(True, alpha=0.3)

    # Add trend line
    z = np.polyfit(x, y, 1)
    p = np.poly1d(z)
    plt.plot(x, p(x), "r--", linewidth=2, label=f"Fit: y = {z[0]:.2f}x + {z[1]:.2f}")
    plt.legend()


@nova.figure("bar-chart")
def plot_bar_chart():
    """Generate a bar chart."""
    categories = ["A", "B", "C", "D", "E"]
    values = [23, 45, 56, 78, 32]
    colors = plt.cm.viridis(np.linspace(0.2, 0.8, len(categories)))

    plt.figure(figsize=(8, 5))
    bars = plt.bar(categories, values, color=colors, edgecolor="black")

    # Add value labels on bars
    for bar, value in zip(bars, values):
        plt.text(
            bar.get_x() + bar.get_width() / 2,
            bar.get_height() + 1,
            str(value),
            ha="center",
            va="bottom",
            fontweight="bold",
        )

    plt.xlabel("Category")
    plt.ylabel("Value")
    plt.title("Category Comparison")
    plt.grid(True, alpha=0.3, axis="y")


@nova.figure("pie-chart")
def plot_pie_chart():
    """Generate a pie chart."""
    labels = ["Python", "Rust", "TypeScript", "Go", "Other"]
    sizes = [35, 25, 20, 15, 5]
    explode = (0.05, 0.05, 0, 0, 0)
    colors = plt.cm.Set3(np.linspace(0, 1, len(labels)))

    plt.figure(figsize=(8, 8))
    plt.pie(
        sizes,
        explode=explode,
        labels=labels,
        colors=colors,
        autopct="%1.1f%%",
        shadow=True,
        startangle=90,
    )
    plt.title("Programming Language Usage")
    plt.axis("equal")
