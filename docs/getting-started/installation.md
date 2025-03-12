# Installation

Setting up Kreuzberg involves installing both the Python package and its system dependencies.

## 1. Install the Python Package

```shell
pip install kreuzberg
```

For optional OCR engines, you can install with extras:

```shell
# For EasyOCR support
pip install "kreuzberg[easyocr]"

# For PaddleOCR support
pip install "kreuzberg[paddleocr]"
```

## 2. Install System Dependencies

Kreuzberg requires two system level dependencies:

- [Pandoc](https://pandoc.org/installing.html) - For document format conversion. Minimum required version is Pandoc 2.
- [Tesseract OCR](https://tesseract-ocr.github.io/) - For image and PDF OCR. Minimum required version is Tesseract 5.

### Linux (Ubuntu/Debian)

```shell
sudo apt-get install pandoc tesseract-ocr
```

For additional language support beyond English:

```shell
# Example: Install German language pack
sudo apt-get install tesseract-ocr-deu
```

### MacOS

```shell
brew install tesseract pandoc
```

For additional language support:

```shell
# Example: Install all language data
brew install tesseract-lang
```

### Windows

Using Chocolatey:

```shell
choco install -y tesseract pandoc
```

## Verifying Installation

You can verify your installation by checking the versions of the installed components:

```shell
# Check Pandoc version
pandoc --version

# Check Tesseract version
tesseract --version
```

## Docker

If you're using Docker, you can include Kreuzberg and its dependencies in your Dockerfile:

```dockerfile
FROM python:3.9-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pandoc \
    tesseract-ocr \
    # Add language packs as needed
    tesseract-ocr-eng \
    && rm -rf /var/lib/apt/lists/*

# Install Kreuzberg
RUN pip install kreuzberg
```

## Notes

- In most distributions, the tesseract-ocr package is split into multiple packages. You may need to install any language models you need aside from English separately.
- Please consult the official documentation for these libraries for the most up-to-date installation instructions for your platform.
