# dfaudit

`dfaudit` is a Dockerfile/Containerfile auditing tool written in Rust.

It recursively searches a directory for container definitions, builds temporary container images using `podman` or `docker`, inspects installed software packages, and generates both machine-readable JSON reports and a human-friendly HTML dashboard with package searching functionality.

The goal is to provide a lightweight way to understand what dependencies exist inside container images. Handy if you are serving JupyterHub with images and the contents are not visable to users.

## Features

* Recursive Dockerfile and Containerfile discovery
* Python package auditing
* R package auditing
* JSON report generation
* HTML report dashboard generation
* Searchable package tables

## How it works

The workflow is:

1. Search the provided directory for:

```
Dockerfile
Containerfile
```

2. Build each container image with Podman.

3. Run audit commands inside the temporary image:

4. Save the results:

```
audit/
├── project-a/
│   └── audit-report.json
├── project-b/
│   └── audit-report.json
└── index.html
```

5. Generate an HTML report containing:

* audited projects
* Python packages
* R packages
* package versions
* report modification times

## Installation

### Requirements

You need:

* Rust
* Cargo
* Podman

### Building dfaudit

Clone the repository:

```bash
git clone https://github.com/taylanot/dfaudit.git
cd dfaudit
```

Build:

```bash
cargo build --release
```

The binary will be available at:

```
target/release/dfaudit
```

### Running

Audit a directory:

```bash
dfaudit --path ./containers
```

Example:

```
containers/
├── application-a/
│   └── Dockerfile
├── application-b/
│   └── Containerfile
```

Output:

```
audit/
├── application-a/
│   └── audit-report.json
├── application-b/
│   └── audit-report.json
└── index.html
```

Open:

```
audit/index.html
```

in your browser.

## Command line options

Example:
```bash
dfaudit \
  --path ./containers \
  --output ./audit \
  --build \
  --engine podman
```

Options:
| Option      | Description                                          |
| ----------- | ----------------------------------------------------- |
| `--path`    | Directory to recursively search                       |
| `--file`    | Audit a single Dockerfile/Containerfile                |
| `--output`  | Output directory                                       |
| `--build`   | Build images or just generate report from json         |
| `--engine`  | Container engine to use: `docker` or `podman`          |
| `--prune`   | Clean up the system after each build                   |
| `--html`    | Generate an HTML report from the output                |
| `--verbose` | Increase logging detail                                |
| `--quiet`   | Reduce output                                          |

## Logging

`dfaudit` uses configurable logging.

Normal mode:
```bash
dfaudit -v --path ./containers
```

Verbose mode:
```bash
dfaudit -vvv --path ./containers
```

The build process can suppress container engine output while still showing application-level progress information.

## Roadmap

Possible future improvements:
* Container vulnerability scanning
* SBOM generation
* Export formats:
  * CSV
  * Markdown
  * PDF
* Historical report comparison
