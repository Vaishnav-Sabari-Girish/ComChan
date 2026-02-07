# Contribution Steps

## OSCG Session Guidelines

1. All Pull Requests coming from OSCG contributors **must include the prefix `OSCG26`** in the PR title.

Example:
OSCG26 fix(parser): handle empty input safely

2. For mentorship, design clarification, flow diagrams, or pre-issue discussion, **use the GitHub Discussions panel first**.

Non-trivial work opened directly as Issues or PRs without prior discussion may be closed or redirected.

---

Follow these steps exactly.
For mentor or admin guidance, use the **Discussions** tab before opening Issues for non-trivial work.

---

## Fork the Repository

Create a fork of the repository under your GitHub account.

---

## Clone Your Fork

```bash
git clone https://github.com/your-username/repo-name.git
cd repo-name
```

---

## Create a New Branch

Do not work on `main` or `master`.

```bash
git switch -c type/short-description
```

Examples:

* feature/add-serial-logging
* fix/null-pointer-crash
* docs/update-readme

---

## Make Changes

* Keep changes focused and aligned with existing code style
* Do not bundle unrelated changes

If the change is large or architectural, **start a Design Discussion before coding**.

---

## Test Your Changes

* Run tests if available
* Add tests when applicable

If tests do not exist, clearly mention this in the PR.

---

## Commit Changes

```bash
git add .
git commit
```

Commit format:

```text
type(scope): short summary
```

Example:

```text
fix(parser): handle empty input safely
```

---

## Push to Your Fork

```bash
git push origin your-branch-name
```

---

## Open a Pull Request

* Target the default branch
* Clearly explain what you changed and why
* Reference related Issues or Discussions

Low-effort or unclear PRs may be closed.

---

## Address Review Feedback

* Respond clearly and professionally
* Push updates to the same branch
* Do not open a new PR unless asked

---

##  Merge

Only maintainers merge PRs.
Do not close or merge your own PR.

If updates are requested, push to the same branch (force push allowed).

---

# Contributor Evaluation

Contributions are evaluated based on:

* Quality
* Correctness
* Clarity
* Usefulness
* Collaboration

Volume alone does not earn points.

---

# OSCG’26 Contributor Guidelines

## Introduction

OSCG is **not a hackathon** and not a race.

The focus is on:

* Sustainable open-source practices
* Clear communication and collaboration
* Real-world workflows

Quality and collaboration matter more than speed.

---

## Accepted Contributions

* Feature improvements
* Bug fixes
* Documentation updates
* Refactoring with clear justification
* Testing and tooling improvements

---

## Not Accepted

* Cosmetic-only changes without purpose
* Copy-pasted or auto-generated code without explanation
* Spam or rushed PRs

**Plagiarism results in immediate removal.**

---

# Communication Guidelines

Use:

* GitHub Issues for bugs and questions
* Pull Request comments for reviews
* Discussions for design and mentor guidance

Avoid:

* Repeated tagging of maintainers
* “Please merge” comments
* Direct messaging admins for PR reviews

Respect async workflows and maintainers’ time.

---

# Reviews and Feedback

* Respond politely and clearly
* Ask if feedback is unclear
* Push fixes to the same PR

Inactive PRs may be closed.

---

# Code of Conduct

All contributors must:

* Communicate respectfully
* Be inclusive and professional
* Respect maintainer decisions

## Zero-Tolerance Behavior

* Harassment or discrimination
* Toxic or aggressive language
* Spam or manipulation
* Misrepresentation of work

Violations may lead to removal from the program.
