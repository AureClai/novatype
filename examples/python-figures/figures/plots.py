"""
Example Python figures for NovaType.

These figures demonstrate how to use the @nova.figure decorator
to generate figures that can be embedded in Typst documents.
"""

import nova
import matplotlib.pyplot as plt
import numpy as np


@nova.figure("sine-wave")
def plot_sine():
    """Generate a simple sine wave plot."""
    x = np.linspace(0, 2 * np.pi, 100)
    y = np.sin(x)

    plt.figure(figsize=(8, 4))
    plt.plot(x, y, "b-", linewidth=2, label=r"$\sin(x)$")
    plt.xlabel("x")
    plt.ylabel("y")
    plt.title("Sine Wave")
    plt.legend()
    plt.grid(True, alpha=0.3)


@nova.figure("cosine-wave")
def plot_cosine():
    """Generate a cosine wave plot."""
    x = np.linspace(0, 2 * np.pi, 100)
    y = np.cos(x)

    plt.figure(figsize=(8, 4))
    plt.plot(x, y, "r-", linewidth=2, label=r"$\cos(x)$")
    plt.xlabel("x")
    plt.ylabel("y")
    plt.title("Cosine Wave")
    plt.legend()
    plt.grid(True, alpha=0.3)


@nova.figure("combined-waves")
def plot_combined():
    """Generate a plot with both sine and cosine."""
    x = np.linspace(0, 2 * np.pi, 100)

    plt.figure(figsize=(10, 5))
    plt.plot(x, np.sin(x), "b-", linewidth=2, label=r"$\sin(x)$")
    plt.plot(x, np.cos(x), "r--", linewidth=2, label=r"$\cos(x)$")
    plt.xlabel("x")
    plt.ylabel("y")
    plt.title("Trigonometric Functions")
    plt.legend()
    plt.grid(True, alpha=0.3)


@nova.figure("histogram")
def plot_histogram():
    """Generate a histogram of random data."""
    np.random.seed(42)
    data = np.random.normal(0, 1, 1000)

    plt.figure(figsize=(8, 5))
    plt.hist(data, bins=30, edgecolor="black", alpha=0.7)
    plt.xlabel("Value")
    plt.ylabel("Frequency")
    plt.title("Normal Distribution (μ=0, σ=1)")
    plt.grid(True, alpha=0.3, axis="y")


@nova.figure("scatter-plot")
def plot_scatter():
    """Generate a scatter plot."""
    np.random.seed(42)
    x = np.random.rand(50)
    y = x + np.random.normal(0, 0.1, 50)
    colors = np.random.rand(50)

    plt.figure(figsize=(8, 6))
    plt.scatter(x, y, c=colors, cmap="viridis", s=100, alpha=0.7)
    plt.colorbar(label="Value")
    plt.xlabel("X")
    plt.ylabel("Y")
    plt.title("Scatter Plot with Color Mapping")
    plt.grid(True, alpha=0.3)
