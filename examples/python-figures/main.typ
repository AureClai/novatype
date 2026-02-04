// Example document demonstrating Python figure integration
//
// To compile this document:
//   nova compile main.typ
//
// This will automatically:
// 1. Execute Python scripts in the figures/ directory
// 2. Generate SVG figures in .nova/cache/
// 3. Compile the Typst document with embedded figures

#import "pyplot.typ": pyplot

#set document(title: "Python Figures Example", author: "NovaType")
#set page(margin: 2cm)
#set text(font: "New Computer Modern")

= Python Figures in NovaType

This document demonstrates how to integrate Python-generated figures
into Typst documents using NovaType.

== Simple Plots

The following figures are generated using matplotlib and numpy:

#figure(
  pyplot("sine-wave", width: 80%),
  caption: [A sine wave generated with Python and matplotlib]
) <fig:sine>

#figure(
  pyplot("cosine-wave", width: 80%),
  caption: [A cosine wave]
) <fig:cosine>

== Combined Plot

Multiple functions can be plotted together:

#figure(
  pyplot("combined-waves", width: 90%),
  caption: [Sine and cosine waves on the same plot]
) <fig:combined>

== Statistical Plots

#figure(
  pyplot("histogram", width: 80%),
  caption: [Histogram of normally distributed data]
) <fig:histogram>

#figure(
  pyplot("scatter-plot", width: 80%),
  caption: [Scatter plot with color mapping]
) <fig:scatter>

== Data-Driven Figures

Figures can depend on external data files. When the data changes,
the figure is automatically regenerated:

#figure(
  pyplot("data-plot", width: 80%),
  caption: [Experimental data with linear fit]
) <fig:data>

== Charts

#figure(
  pyplot("bar-chart", width: 70%),
  caption: [Bar chart comparing categories]
) <fig:bar>

#figure(
  pyplot("pie-chart", width: 60%),
  caption: [Pie chart showing language usage]
) <fig:pie>

== Inline Usage

You can also use `pyplot` inline without a figure wrapper:

Here's a quick preview of the sine wave: #pyplot("sine-wave", width: 30%)

== How It Works

=== Python Side

Define figures in the `figures/` directory using the `@nova.figure` decorator:

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

=== Typst Side

Use the `pyplot()` function to embed figures:

```typst
#import "@local/nova:0.1.0": pyplot

#figure(
  pyplot("sine-wave", width: 80%),
  caption: [A sine wave]
) <fig:sine>
```

=== Compilation

When you run `nova compile main.typ`, NovaType:

1. Discovers all `@nova.figure` decorated functions
2. Checks if figures need regeneration (based on source hash)
3. Executes Python to generate SVG files
4. Compiles the Typst document

This separation of concerns keeps your `.typ` files clean and allows
proper code reuse in Python.
