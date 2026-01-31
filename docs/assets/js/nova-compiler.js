/**
 * NovaType WASM Compiler - Browser Integration
 *
 * This module provides a simple interface to the NovaType WASM compiler
 * for live document compilation in the browser.
 */

// Global state
let wasmModule = null;
let compiler = null;
let isLoading = false;
let loadError = null;
const loadCallbacks = [];

/**
 * Initialize the NovaType WASM compiler.
 * Returns a promise that resolves when the compiler is ready.
 */
export async function initNovaCompiler() {
  // Already loaded
  if (compiler) {
    return compiler;
  }

  // Currently loading - wait for it
  if (isLoading) {
    return new Promise((resolve, reject) => {
      loadCallbacks.push({ resolve, reject });
    });
  }

  // Start loading
  isLoading = true;

  try {
    // Dynamic import of the WASM module
    const wasmPath = getWasmBasePath();
    const { default: init, NovaCompiler } = await import(`${wasmPath}/nova_wasm.js`);

    // Initialize WASM
    await init(`${wasmPath}/nova_wasm_bg.wasm`);

    // Create compiler instance
    compiler = new NovaCompiler();
    compiler.setStrictValidation(false);

    wasmModule = { init, NovaCompiler };

    // Resolve all waiting promises
    loadCallbacks.forEach(({ resolve }) => resolve(compiler));
    loadCallbacks.length = 0;

    console.log('%c NovaType WASM %c Compiler ready!',
      'background: #2563eb; color: white; padding: 2px 6px; border-radius: 3px;',
      'color: #10b981;'
    );

    return compiler;
  } catch (error) {
    loadError = error;
    console.error('Failed to load NovaType WASM:', error);

    // Reject all waiting promises
    loadCallbacks.forEach(({ reject }) => reject(error));
    loadCallbacks.length = 0;

    throw error;
  } finally {
    isLoading = false;
  }
}

/**
 * Get the base path for WASM files.
 */
function getWasmBasePath() {
  // Determine the base path based on current location
  const path = window.location.pathname;

  // Extract the base path (e.g., /novatype/ on GitHub Pages)
  const basePath = path.substring(0, path.lastIndexOf('/'));

  if (path.includes('/docs/')) {
    // Inside /docs/ subfolder - go up one level
    return basePath.replace(/\/docs$/, '') + '/assets/wasm';
  }
  // Root level (demos.html, index.html)
  return basePath + '/assets/wasm';
}

/**
 * Compile Typst source to SVG.
 *
 * @param {string} source - The Typst source code
 * @returns {Promise<CompileResult>} The compilation result
 */
export async function compile(source) {
  const comp = await initNovaCompiler();

  try {
    const result = comp.compile(source);
    return {
      success: result.success,
      svg_pages: result.svg_pages || [],
      errors: result.errors || [],
      metadata: result.metadata,
      content: result.content
    };
  } catch (error) {
    return {
      success: false,
      svg_pages: [],
      errors: [error.message || String(error)],
      metadata: null,
      content: source
    };
  }
}

/**
 * Compile Typst source and return only the first page SVG.
 *
 * @param {string} source - The Typst source code
 * @returns {Promise<string>} The SVG string
 */
export async function compileToSvg(source) {
  const comp = await initNovaCompiler();
  return comp.compileToSvg(source);
}

/**
 * Check if the compiler is loaded.
 */
export function isCompilerReady() {
  return compiler !== null;
}

/**
 * Get the loading error if any.
 */
export function getLoadError() {
  return loadError;
}

/**
 * Create a live compiler instance for a specific demo.
 * Handles debouncing and error display.
 */
export class LiveCompiler {
  constructor(options = {}) {
    this.inputElement = options.input;
    this.outputElement = options.output;
    this.errorElement = options.error;
    this.loadingElement = options.loading;
    this.debounceMs = options.debounce || 300;
    this.onCompile = options.onCompile;
    this.onError = options.onError;

    this._debounceTimer = null;
    this._lastSource = null;
  }

  /**
   * Initialize the live compiler with event listeners.
   */
  async init() {
    // Show loading state
    this._setLoading(true);

    try {
      await initNovaCompiler();
      this._setLoading(false);

      // Initial compile
      if (this.inputElement) {
        this.compile(this.inputElement.value);

        // Set up input listener
        this.inputElement.addEventListener('input', () => {
          this._scheduleCompile();
        });
      }
    } catch (error) {
      this._setLoading(false);
      this._showError(`Failed to load compiler: ${error.message}`);
    }
  }

  /**
   * Compile the current input.
   */
  async compile(source) {
    if (source === this._lastSource) {
      return;
    }
    this._lastSource = source;

    try {
      const result = await compile(source);

      if (result.success && result.svg_pages.length > 0) {
        this._showResult(result.svg_pages[0]);
        this._hideError();

        if (this.onCompile) {
          this.onCompile(result);
        }
      } else if (result.errors.length > 0) {
        this._showError(result.errors.join('\n'));

        if (this.onError) {
          this.onError(result.errors);
        }
      }
    } catch (error) {
      this._showError(error.message || String(error));

      if (this.onError) {
        this.onError([error.message]);
      }
    }
  }

  /**
   * Schedule a compile with debouncing.
   */
  _scheduleCompile() {
    if (this._debounceTimer) {
      clearTimeout(this._debounceTimer);
    }

    this._debounceTimer = setTimeout(() => {
      if (this.inputElement) {
        this.compile(this.inputElement.value);
      }
    }, this.debounceMs);
  }

  /**
   * Show the SVG result.
   */
  _showResult(svg) {
    if (this.outputElement) {
      this.outputElement.innerHTML = svg;

      // Scale SVG to fit container
      const svgEl = this.outputElement.querySelector('svg');
      if (svgEl) {
        svgEl.style.width = '100%';
        svgEl.style.height = 'auto';
        svgEl.style.maxHeight = '500px';
      }
    }
  }

  /**
   * Show an error message.
   */
  _showError(message) {
    if (this.errorElement) {
      this.errorElement.textContent = message;
      this.errorElement.style.display = 'block';
    }
  }

  /**
   * Hide the error message.
   */
  _hideError() {
    if (this.errorElement) {
      this.errorElement.style.display = 'none';
    }
  }

  /**
   * Set loading state.
   */
  _setLoading(loading) {
    if (this.loadingElement) {
      this.loadingElement.style.display = loading ? 'flex' : 'none';
    }
  }
}

// Export for global access
window.NovaCompiler = {
  init: initNovaCompiler,
  compile,
  compileToSvg,
  isReady: isCompilerReady,
  getError: getLoadError,
  LiveCompiler
};
