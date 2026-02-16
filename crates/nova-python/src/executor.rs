//! Python subprocess execution for figure generation.

use crate::config::PythonConfig;
use crate::error::{Error, Result};
use crate::registry::Figure;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Executes Python code to generate figures.
pub struct PythonExecutor {
    config: PythonConfig,
}

impl PythonExecutor {
    /// Create a new executor with the given configuration.
    pub fn new(config: &PythonConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Check if Python is available and nova package is installed.
    pub async fn check_environment(&self) -> Result<()> {
        let python = self.config.python_executable();

        // Check Python exists
        let output = Command::new(&python)
            .arg("--version")
            .output()
            .await
            .map_err(|_| Error::PythonNotFound {
                path: python.display().to_string(),
            })?;

        if !output.status.success() {
            return Err(Error::PythonNotFound {
                path: python.display().to_string(),
            });
        }

        // Check nova package is installed
        let output = Command::new(&python)
            .args(["-c", "import nova; print(nova.__version__)"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(Error::NovaPackageNotInstalled);
        }

        Ok(())
    }

    /// Execute a figure function and capture the SVG output.
    pub async fn execute_figure(&self, figure: &Figure) -> Result<PathBuf> {
        let python = self.config.python_executable();

        // Generate the runner script
        let runner_script = self.generate_runner_script(figure);

        tracing::debug!(
            figure = %figure.name,
            function = %figure.function_name,
            "Executing Python figure"
        );

        tracing::debug!(script = %runner_script, "Generated Python script");
        tracing::debug!(pythonpath = %self.config.build_python_path(), "PYTHONPATH");
        tracing::debug!(cwd = %self.config.project_root.display(), "Working directory");

        // Run Python with the script
        let child = Command::new(&python)
            .arg("-c")
            .arg(&runner_script)
            .current_dir(&self.config.project_root)
            .env("PYTHONPATH", self.config.build_python_path())
            .envs(&self.config.env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Wait with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.timeout),
            child.wait_with_output(),
        )
        .await
        .map_err(|_| Error::ExecutionFailed {
            message: format!(
                "Figure '{}' timed out after {}s",
                figure.name, self.config.timeout
            ),
        })??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        tracing::debug!(
            status = %output.status,
            stdout = %stdout,
            stderr = %stderr,
            "Python execution result"
        );

        if !output.status.success() {
            let traceback = crate::error::PythonTraceback::parse(&stderr).map(Box::new);
            return Err(Error::ScriptError {
                file: figure.source_file.clone(),
                error: stderr.to_string(),
                traceback,
            });
        }

        // The script outputs the SVG path on stdout
        let svg_path = stdout.trim().to_string();

        if svg_path.is_empty() {
            return Err(Error::GenerationFailed {
                name: figure.name.clone(),
                reason: "No SVG output produced".to_string(),
            });
        }

        Ok(PathBuf::from(svg_path))
    }

    /// Execute all figures in a Python file.
    pub async fn execute_all(&self, figures: &[Figure]) -> Result<Vec<(String, PathBuf)>> {
        let mut results = Vec::new();

        for figure in figures {
            let svg_path = self.execute_figure(figure).await?;
            results.push((figure.name.clone(), svg_path));
        }

        Ok(results)
    }

    /// Generate the Python script that runs a figure function.
    fn generate_runner_script(&self, figure: &Figure) -> String {
        let module_path = self.figure_module_path(figure);
        // Escape backslashes for Windows paths in Python strings
        let cache_dir = self
            .config
            .cache_dir
            .display()
            .to_string()
            .replace('\\', "\\\\");
        let figure_name = &figure.name;
        let function_name = &figure.function_name;

        format!(
            r#"
import sys
import os

# Ensure cache directory exists
os.makedirs("{cache_dir}", exist_ok=True)

# Import the module containing the figure
{module_import}

# Import nova to set up the figure context
import nova

# Set output path
output_path = os.path.join("{cache_dir}", "{figure_name}.svg")

# Execute the figure function with nova context
nova._execute_figure({module_ref}.{function_name}, output_path)

# Print the output path
print(output_path)
"#,
            cache_dir = cache_dir,
            figure_name = figure_name,
            function_name = function_name,
            module_import = module_path.0,
            module_ref = module_path.1,
        )
    }

    /// Convert a file path to a Python module import statement.
    fn figure_module_path(&self, figure: &Figure) -> (String, String) {
        // Convert path like "figures/analysis/plots.py" to "figures.analysis.plots"
        let relative_path = figure
            .source_file
            .strip_prefix(&self.config.figures_dir)
            .unwrap_or(&figure.source_file);

        let path_without_ext = relative_path.with_extension("");
        let module_parts: Vec<String> = path_without_ext
            .components()
            .filter_map(|c| c.as_os_str().to_str().map(|s| s.to_string()))
            .collect();

        // Get the figures directory name as base module
        let base_module = self
            .config
            .figures_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("figures");

        let full_module = if module_parts.is_empty() {
            base_module.to_string()
        } else {
            format!("{}.{}", base_module, module_parts.join("."))
        };

        // Create import statement and module reference
        let import_stmt = format!("import {}", full_module);
        let module_ref = full_module;

        (import_stmt, module_ref)
    }
}

/// Generate all figures using the nova Python runner.
///
/// This is a higher-level function that uses nova's built-in runner
/// to generate all figures at once, which is more efficient than
/// running each figure separately.
pub async fn generate_all_figures(config: &PythonConfig) -> Result<Vec<(String, PathBuf)>> {
    let python = config.python_executable();
    let cache_dir = &config.cache_dir;
    let figures_dir = &config.figures_dir;

    // Ensure directories exist
    std::fs::create_dir_all(cache_dir)?;

    let script = format!(
        r#"
import nova
nova.generate_all("{figures_dir}", "{cache_dir}")
"#,
        figures_dir = figures_dir.display(),
        cache_dir = cache_dir.display(),
    );

    let output = Command::new(&python)
        .arg("-c")
        .arg(&script)
        .current_dir(&config.project_root)
        .env("PYTHONPATH", config.build_python_path())
        .envs(&config.env)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::ExecutionFailed {
            message: stderr.to_string(),
        });
    }

    // Parse output to get generated figure paths
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        if let Some((name, path)) = line.split_once(':') {
            results.push((name.trim().to_string(), PathBuf::from(path.trim())));
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_path_conversion() {
        let config = PythonConfig {
            figures_dir: PathBuf::from("/project/figures"),
            ..Default::default()
        };
        let executor = PythonExecutor::new(&config).unwrap();

        let figure = Figure::new("test", "/project/figures/analysis/plots.py", "make_plot");

        let (import_stmt, module_ref) = executor.figure_module_path(&figure);

        assert!(import_stmt.contains("import"));
        assert!(module_ref.contains("figures"));
    }

    #[test]
    fn test_runner_script_generation() {
        let config = PythonConfig {
            figures_dir: PathBuf::from("/project/figures"),
            cache_dir: PathBuf::from("/project/.nova/cache"),
            project_root: PathBuf::from("/project"),
            ..Default::default()
        };
        let executor = PythonExecutor::new(&config).unwrap();

        let figure = Figure::new("my-fig", "/project/figures/plots.py", "plot_data");
        let script = executor.generate_runner_script(&figure);

        assert!(script.contains("nova._execute_figure"));
        assert!(script.contains("my-fig.svg"));
        assert!(script.contains("plot_data"));
    }
}
