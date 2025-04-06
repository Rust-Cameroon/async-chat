# How to Contribute 

Thank you for your interest in contributing to our async chatting app project! Below are the guidelines to help you get started with contributing effectively.

## Submit Pull Requests

- All changes should be proposed via GitHub pull requests.

- A project maintainer will review your PR, provide feedback, and either approve or request modifications.

- Even experienced contributors should follow this process.

## Claim Issues First

Before working on an issue, comment on it to let others know you're tackling it.This prevents duplicate work and helps coordinate efforts.

## Reporting New Issues

Found a bug or have a feature idea?

- Search existing issues to avoid duplicates.

- Use templates: Fill out the Bug Report or Feature Request template.

- Provide details:

    - Bug: Steps to reproduce, expected vs. actual behavior.

    - Feature: Use case and proposed solution.



## Get Started
Fork the repository on GitHub

Clone your fork locally:

```sh
git clone https://github.com/your-username/async-chat-app.git
cd async-chat-app
```

Set up upstream remote:
```sh
git remote add upstream https://github.com/original-owner/async-chat-app.git
```


## Branch Naming

Use descriptive branch names prefixed with:

  - feature/issue-1-local-chat

  - bugfix/issue-3-linting-workflow

  - docs/issue-7-contribution-guidelines

  - refactor/ issue-number-changes

  - test/issue-number-changes

## Commit Messages

  - Follow Conventional Commits style
  
  - Feat(user): integrate user management into server connection handling

  - Use the present tense ("Add feature" not "Added feature")

  - Keep messages concise but descriptive

## Pull Requests

- Keep PRs focused on a single feature/bugfix

- Reference any related issues

- Include a clear description of changes

- Update documentation if needed

- Ensure all tests pass

## Pull Request Checklist

Before submitting your pull request, please ensure the following:

- Branch from main: Create your feature/bugfix branch from the latest main branch. If updates occur in main while your PR is pending, rebase your changes to avoid merge conflicts.

- Keep commits small & functional: Each commit should be self-contained (compiling and passing tests) while being as granular as possible.

- Sign-off commits: Include a Developer Certificate of Origin (DCO) sign-off using git commit -s to certify your contribution under the project’s license terms.

- Request reviews proactively: If your PR needs attention, tag relevant reviewers (@username) in a comment or reach out in the project’s chat (e.g., Discord channel).

- Add tests: Include unit/integration tests for new features or bug fixes. For backend changes, test API endpoints; for frontend, add UI/component tests as needed. Refer to the testing guide (link your project’s guide here) for specifics.

Note: PRs that don’t merge cleanly with main may require a rebase before approval.

## Code Style
### General

- Follow existing patterns in the codebase

- Keep functions small and focused

- Use descriptive variable names



## Testing

Write tests for new features

Update tests when fixing bugs

Run all tests before submitting PR:
```bash
    cargo test
```

## Communication

- Use GitHub issues for feature requests and bug reports

- Be respectful and inclusive in all communications

- Ask questions if anything is unclear

## Getting Help

If you need help at any point:

- Check the project's [README](./README.md)

- Look through existing issues

- Reach out to maintainers via GitHub discussions

We appreciate your contributions and look forward to collaborating with you!