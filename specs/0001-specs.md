# Specs Directory Guide

## Overview

The `specs` directory is used to record and maintain all development standards and specifications for this codebase. All specification files are written in Markdown format for easy reading and maintenance.

## File Naming Convention

- All files must be named using the `NNNN-filename.md` format
- `NNNN` is a four-digit number (0000-9999) used to specify file priority and reading order
- `filename` is a descriptive English filename using lowercase letters and hyphens
- File extension must be `.md`

### Examples

- `0000-specs-guide.md` - This file, describing the basic specifications for the specs directory
- `0001-coding-standards.md` - Coding standards
- `0002-architecture.md` - Architecture design specifications
- `0010-testing.md` - Testing specifications

## Content Requirements

Each specification file should include:

1. **Title**: Clear and descriptive title
2. **Overview**: Scope and purpose of the specification
3. **Detailed Specifications**: Specific rules and requirements
4. **References**: Links to related documentation or resources (if applicable)

### Notes on Examples and Code Snippets

- **Examples are optional**: Specification files do not need to provide full, runnable examples. Clear logical definitions and structure descriptions are sufficient.
- **Structs and types**: When describing data structures, it is enough to specify the fields, types, and invariants. You do not need to show all helper methods unless they are part of the public contract.
- **Functions and methods**:
  - Always include the function signature (name, parameters, return type, and error model).
  - The function body should only show the core logic or key steps when necessary to clarify behavior.
  - Omit boilerplate and non-essential details (logging, error wrapping, etc.) from the spec; those belong in implementation, not in specifications.

## Maintenance Principles

- All specification files should be kept up to date to reflect the current state of the codebase
- When specifications change, relevant files should be updated promptly
- When adding new specifications, choose an appropriate number and create a new file
- Deleted or deprecated specifications should be clearly marked in the file

## Language Requirements

- All specification files must be written in English only
- Use clear and concise English prose
- Technical terms should be used consistently throughout the documentation
- **Avoid abbreviations for common terms**: Use full names instead of short forms in specification text and identifiers (e.g. use **transaction** rather than **tx**, **address** rather than **addr**). This improves clarity and consistency across specs.
- **Established identifiers keep their names**: Compound or legacy names that are already in use (e.g. `txhash`, `created_txhash`, `deleted_txhash`) remain as-is; do not expand them to full form (e.g. do not rename `txhash` to `transaction_hash`).

