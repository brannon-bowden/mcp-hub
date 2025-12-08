# Contributing to MCP Hub

Thank you for your interest in contributing to MCP Hub! This document provides guidelines and best practices for contributing to this project.

## Table of Contents

- [Commit Message Guidelines](#commit-message-guidelines)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Branch Naming Convention](#branch-naming-convention)
- [Code Style](#code-style)

## Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification for commit messages. This leads to more readable messages that are easy to follow when looking through the project history.

### Commit Message Format

Each commit message consists of a **header**, an optional **body**, and an optional **footer**:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type

Must be one of the following:

| Type | Description |
|------|-------------|
| **feat** | A new feature |
| **fix** | A bug fix |
| **docs** | Documentation only changes |
| **style** | Changes that do not affect the meaning of the code (white-space, formatting, etc.) |
| **refactor** | A code change that neither fixes a bug nor adds a feature |
| **perf** | A code change that improves performance |
| **test** | Adding missing tests or correcting existing tests |
| **build** | Changes that affect the build system or external dependencies |
| **ci** | Changes to our CI configuration files and scripts |
| **chore** | Other changes that don't modify src or test files |
| **revert** | Reverts a previous commit |

### Scope (Optional)

The scope should be the name of the module affected (e.g., `proxy`, `ui`, `database`, `settings`, `servers`).

### Subject

The subject contains a succinct description of the change:

- Use the imperative, present tense: "change" not "changed" nor "changes"
- Don't capitalize the first letter
- No period (.) at the end
- Keep it under 72 characters

### Body (Optional)

The body should include the motivation for the change and contrast this with previous behavior. Use the imperative, present tense.

### Footer (Optional)

The footer should contain any information about **Breaking Changes** and is also the place to reference GitHub issues that this commit closes.

### Examples

**Good commit messages:**

```
feat(proxy): add HTTP/SSE server for MCP aggregation

Implements core infrastructure for MCP Hub to act as a proxy/aggregator.
Clients can connect via a single HTTP/SSE endpoint.

- Add proxy service module with axum
- Add MCP server process management
- Add JSON-RPC message routing with tool namespacing
```

```
fix(ui): resolve sync status not updating when toggling servers

The setServerEnabled function was updating the local enabledServers
array but not the lastModified timestamp, causing the "needs sync"
indicator to not appear after changes.
```

```
docs: update README with comprehensive feature documentation
```

```
style(ui): improve checkbox visibility with darker border
```

**Bad commit messages (avoid these):**

- ❌ `Fix` - Too vague, no context
- ❌ `Fixes` - No indication of what was fixed
- ❌ `Try this` - Unprofessional, no context
- ❌ `WIP` - Should not be in main branch
- ❌ `Update file.ts` - Doesn't explain why

## Pull Request Guidelines

### PR Title

PR titles should follow the same format as commit messages:

```
<type>(<scope>): <description>
```

Examples:
- `feat(proxy): add MCP proxy server with HTTP/SSE support`
- `fix(database): resolve migration issue with parent_id column`
- `docs: add API documentation for proxy endpoints`

### PR Description

Every PR should include a description with the following sections:

```markdown
## Summary

Brief description of what this PR does.

## Changes

- List of specific changes made
- Another change
- And another

## Testing

How was this tested? Include steps to reproduce or verify the changes.

## Screenshots (if applicable)

Include screenshots for UI changes.

## Related Issues

Fixes #123
Related to #456
```

### PR Checklist

Before submitting a PR, ensure:

- [ ] Code compiles without errors
- [ ] All tests pass
- [ ] Commit messages follow the guidelines
- [ ] PR title follows the guidelines
- [ ] PR description is complete
- [ ] Documentation is updated (if applicable)

## Branch Naming Convention

Use the following format for branch names:

```
<type>/<short-description>
```

Examples:
- `feat/proxy-server`
- `fix/sync-status-update`
- `docs/contributing-guide`
- `refactor/database-schema`

## Code Style

### TypeScript/React (Frontend)

- Use functional components with hooks
- Use TypeScript strict mode
- Follow ESLint configuration
- Use Tailwind CSS for styling

### Rust (Backend)

- Follow Rust standard formatting (`cargo fmt`)
- Use Clippy for linting (`cargo clippy`)
- Document public APIs with doc comments

---

Thank you for contributing to MCP Hub! 🚀
