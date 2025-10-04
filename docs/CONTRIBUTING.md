# Contributing to Devalang

First off, thank you for considering contributing to Devalang! 🎉

It's people like you that make Devalang such a great tool. We welcome contributions from everyone, whether it's:

- 🐛 Bug reports
- 💡 Feature requests
- 📝 Documentation improvements
- 🎵 Example contributions
- 🔧 Code contributions
- 🌍 Translations

---

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Submitting Changes](#submitting-changes)
- [Coding Guidelines](#coding-guidelines)
- [Testing](#testing)
- [Documentation](#documentation)
- [Community](#community)

---

## 📜 Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to contact@devaloop.com.

### Our Standards

- ✅ Be respectful and inclusive
- ✅ Welcome newcomers and help them learn
- ✅ Focus on what is best for the community
- ✅ Show empathy towards other community members

---

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.70+ with Cargo
- **Node.js** 16+ with npm
- **Git** for version control
- **Code editor** (VS Code recommended)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:

```bash
git clone https://github.com/YOUR_USERNAME/devalang.git
cd devalang
```

3. Add upstream remote:

```bash
git remote add upstream https://github.com/devaloop-labs/devalang.git
```

### Install Dependencies

```bash
# Install Rust dependencies
cargo build

# Install Node.js dependencies
npm install

# Build TypeScript
npm run ts:build
```

### Verify Setup

```bash
# Run tests
cargo test --features cli
npm test

# Check code formatting
cargo fmt --check
```

---

## 🛠️ Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

Branch naming conventions:
- `feature/` for new features
- `fix/` for bug fixes
- `docs/` for documentation
- `test/` for test additions
- `refactor/` for code refactoring

### 2. Make Changes

#### Rust Development

```bash
# Build the project
cargo build --features cli

# Run with your changes
cargo run --features cli -- build --path examples/simple.deva

# Format code
cargo fmt

# Run clippy for linting
cargo clippy --features cli -- -D warnings
```

#### TypeScript Development

```bash
# Build TypeScript
npm run ts:build

# Watch mode
npm run ts:watch

# Test your changes
npm test
```

### 3. Test Your Changes

```bash
# Run all Rust tests
cargo test --features cli,wasm

# Run TypeScript tests
npm test

# Test on specific examples
cargo run --features cli -- check --entry examples/
```

### 4. Commit Your Changes

We use [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git add .
git commit -m "feat: add new synth oscillator type"
# or
git commit -m "fix: resolve pattern parsing issue"
# or
git commit -m "docs: improve README examples"
```

Commit types:
- `feat:` new feature
- `fix:` bug fix
- `docs:` documentation only
- `style:` formatting, missing semicolons, etc.
- `refactor:` code refactoring
- `test:` adding tests
- `chore:` maintenance

---

## 📤 Submitting Changes

### Pull Request Process

1. **Update your fork**:
```bash
git fetch upstream
git rebase upstream/main
```

2. **Push to your fork**:
```bash
git push origin feature/your-feature-name
```

3. **Create Pull Request**:
   - Go to GitHub and create a PR from your branch
   - Fill out the PR template completely
   - Link any related issues

### PR Title Format

```
<type>: <short description>
```

Examples:
- `feat: add MP3 export support`
- `fix: resolve WASM bank injection error`
- `docs: add synth examples to README`

### PR Description

Include:
- **What**: Describe the changes
- **Why**: Explain the motivation
- **How**: Outline the approach
- **Testing**: Describe how you tested
- **Screenshots**: If UI changes (optional)

### Review Process

- Maintainers will review your PR
- Address feedback and requested changes
- Once approved, your PR will be merged
- Your contribution will be credited in the changelog

---

## 💻 Coding Guidelines

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Add documentation comments (`///`) for public APIs
- Keep functions focused and small
- Use descriptive variable names

Example:
```rust
/// Renders audio from DevaLang code
/// 
/// # Arguments
/// * `code` - The DevaLang source code
/// * `options` - Rendering options (sample rate, BPM)
/// 
/// # Returns
/// * `Result<Vec<f32>>` - Audio buffer or error
pub fn render_audio(code: &str, options: RenderOptions) -> Result<Vec<f32>> {
    // Implementation
}
```

### TypeScript Style

- Use TypeScript strict mode
- Follow ESLint rules
- Use meaningful variable names
- Add JSDoc comments for exported functions
- Prefer `const` over `let`
- Use async/await over callbacks

Example:
```typescript
/**
 * Parse DevaLang code to AST
 * @param code - DevaLang source code
 * @returns Parsed AST or error
 */
export async function parse(code: string): Promise<AST> {
  // Implementation
}
```

### Devalang Code Style

- Use 4 spaces for indentation
- Add comments for complex logic
- Group related statements
- Use descriptive variable names

Example:
```deva
# Set tempo
bpm 120

# Load bank
bank devaloop.808 as drums

# Create pattern
pattern kick with drums.kick = "x--- x--- x--- x---"
```

---

## 🧪 Testing

### Writing Tests

#### Rust Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempo_validation() {
        let result = execute_tempo(120.0);
        assert!(result.is_ok());
    }
}
```

#### TypeScript Tests

```typescript
describe('render', () => {
  it('should render audio from code', async () => {
    const code = 'bpm 120';
    const result = await render(code);
    expect(result).toBeDefined();
  });
});
```

### Running Tests

```bash
# Run all tests
cargo test --features cli,wasm
npm test

# Run specific test
cargo test test_tempo_validation
npm test -- --grep "render"

# Run with coverage (if configured)
cargo tarpaulin
npm run test:coverage
```

---

## 📚 Documentation

### Where to Document

- **Code comments**: For implementation details
- **Doc comments**: For public APIs
- **README.md**: For getting started
- **CHANGELOG.md**: For version changes
- **Examples**: For usage demonstrations

### Documentation Style

- Be clear and concise
- Include examples
- Explain the "why" not just the "what"
- Keep it up to date

### Adding Examples

1. Create a `.deva` file in `examples/`
2. Add comments explaining the code
3. Test it works: `devalang check --entry examples/your-example.deva`
4. Document it in README if significant

---

## 🎯 Issue Guidelines

### Reporting Bugs

Use the bug report template and include:
- **Environment**: OS, Rust version, Node version
- **Steps to reproduce**: Minimal example
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Error messages**: Copy-paste full errors
- **Code sample**: Minimal DevaLang code that triggers the bug

### Feature Requests

Use the feature request template and include:
- **Problem**: What problem does this solve?
- **Solution**: Describe your proposed solution
- **Alternatives**: What alternatives have you considered?
- **Examples**: Show example usage

---

## 💬 Community

### Communication Channels

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and ideas
- **Discord**: For real-time chat (coming soon)
- **Email**: contact@devaloop.com for private inquiries

### Getting Help

- Check the [documentation](https://docs.devalang.com)
- Search existing issues
- Ask in GitHub Discussions
- Join our Discord community

---

## 🏆 Recognition

Contributors are recognized in:
- CHANGELOG.md for each release
- GitHub contributors page
- Project README (for significant contributions)

---

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

## ❓ Questions?

Don't hesitate to ask! We're here to help:
- Open a [GitHub Discussion](https://github.com/devaloop-labs/devalang/discussions)
- Email us at contact@devaloop.com
- Check our [FAQ](https://docs.devalang.com/faq)

---

<div align="center">
    <strong>Thank you for contributing to Devalang! 🦊</strong>
    <br />
    <sub>Every contribution makes a difference</sub>
</div>
