# dfaudit

`dfaudit` is a Dockerfile/Containerfile auditing tool written in Rust.

It recursively searches a directory for container definitions, builds temporary container images using Podman, inspects installed software packages, and generates both machine-readable JSON reports and a human-friendly HTML dashboard.

The goal is to provide a lightweight way to understand what dependencies exist inside container images.

## Features

* Recursive Dockerfile and Containerfile discovery
* Podman-based container builds
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

Python:

```bash
python3 -m pip list --format=json
```

R:

```bash
Rscript -e "installed.packages()[,c('Package','Version')]"
```

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

## Requirements

You need:

* Rust
* Cargo
* Podman

## Building dfaudit

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

## Running

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
  --output ./audit
```

Options:

| Option      | Description                             |
| ----------- | --------------------------------------- |
| `--path`    | Directory to recursively search         |
| `--file`    | Audit a single Dockerfile/Containerfile |
| `--output`  | Output directory                        |
| `--verbose` | Increase logging detail                 |
| `--quiet`   | Reduce output                           |

## Logging

`dfaudit` uses configurable logging.

Normal mode:

```bash
dfaudit --path ./containers
```

Verbose mode:

```bash
dfaudit --verbose --path ./containers
```

Quiet mode:

```bash
dfaudit --quiet --path ./containers
```

The build process can suppress Podman output while still showing application-level progress information.

## Roadmap

Possible future improvements:

* Docker support
* Container vulnerability scanning
* SBOM generation
* Package license reporting
* Export formats:
  * CSV
  * Markdown
  * PDF
* Historical report comparison
