"""Minimal figure for integration testing."""

import nova
import matplotlib.pyplot as plt


@nova.figure("test-circle")
def plot_circle():
    """Generate a minimal test figure."""
    plt.figure(figsize=(2, 2))
    plt.plot([0], [0], "ro", markersize=10)
    plt.title("Test")
