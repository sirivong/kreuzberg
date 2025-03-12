# Contributing to Kreuzberg

Thank you for considering contributing to Kreuzberg! This document provides guidelines and instructions for contributing to the project.

## Development Setup

1. Clone the repository:

    ```bash
    git clone https://github.com/Goldziher/kreuzberg.git
    cd kreuzberg
    ```

1. Create and activate a virtual environment:

    ```bash
    python -m venv .venv
    source .venv/bin/activate  # On Windows: .venv\Scripts\activate
    ```

1. Install development dependencies:

    ```bash
    pip install -e ".[dev,docs]"
    ```

## Running Tests

```bash
pytest
```

## Code Style

This project uses:

- [Black](https://github.com/psf/black) for code formatting
- [isort](https://pycqa.github.io/isort/) for import sorting
- [mypy](http://mypy-lang.org/) for static type checking
- [ruff](https://github.com/charliermarsh/ruff) for linting

You can run all style checks with:

```bash
black .
isort .
mypy .
ruff check .
```

## Documentation

Documentation is built with [MkDocs](https://www.mkdocs.org/) using the [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/) theme.

To build and serve the documentation locally:

```bash
mkdocs serve
```

## Pull Request Process

1. Fork the repository
1. Create a feature branch (`git checkout -b feature/amazing-feature`)
1. Commit your changes (`git commit -m 'Add some amazing feature'`)
1. Push to the branch (`git push origin feature/amazing-feature`)
1. Open a Pull Request

## License

By contributing to Kreuzberg, you agree that your contributions will be licensed under the project's MIT License.
