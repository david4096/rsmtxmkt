# rsmtxmkt

## Overview

`rsmtxmkt` is a Rust library and Python extension for loading and processing sparse matrices in the Matrix Market format. It utilizes Rust's performance and safety features while providing Python bindings for easy integration into Python projects.

## Features

- **Asynchronous File Loading**: Leverages Rust's `tokio` for asynchronous file reading.
- **Compressed Sparse Matrix Format**: Converts Matrix Market files into the Compressed Sparse Row (CSR) format using the `sprs` crate.
- **Python Bindings**: Exposes functionality to Python, allowing for seamless use in Python scripts and projects.

