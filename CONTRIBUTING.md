# Contributing guidelines

By contributing to `Device SDK for Spotflow IoT Platform`, you declare that:

* You are entitled to assign the copyright for the work, provided it is not owned by your employer or you have received a written copyright assignment.
* You license your contribution under the same terms that apply to the rest of the `Device SDK for Spotflow IoT Platform` project.
* You pledge to follow the [Code of Conduct](./CODE_OF_CONDUCT.md).

## Contribution process

Please, always create an [Issue](https://github.com/spotflow-io/device-sdk/issues/new) before starting to work on a new feature or bug fix. This way, we can discuss the best approach and avoid duplicated or lost work. Without discussing the issue first, there is a risk that your PR will not be accepted because e.g.:

* It does not fit the project's goals.
* It is not implemented in the way that we would like to see.
* It is already being worked on by someone else.

### Commits & Pull Requests

We do not put any specific requirements on individual commits. However, we expect that the Pull Request (PR) is a logical unit of work that is easily understandable & reviewable. The PRs should also contain expressive title and description.

Few general rules are:

* Do not mix multiple unrelated changes in a single PR.
* Do not mix formatting changes with functional changes.
* Do not mix refactoring with functional changes.
* Do not create huge PRs that are hard to review. In case that your change is logically cohesive but still large, consider splitting it into multiple PRs.

### Code style

This code uses `cargo fmt` to format the code and `cargo clippy` to improve the code quality.
Both are checked in the CI pipeline and failing to pass the formatting or linting check will result in a failed build.

### Testing

All new code must be covered by tests. Prefer adding new tests specific for the new code, but feel free to extend existing tests if it makes sense.

### Documentation & changelog

All new features and changes must be reflected in [README.md](./README.md) and/or documentation comments of the public interface. Also, make sure to update the appropriate `CHANGELOG.md` file ([Rust](./spotflow/CHANGELOG.md), [Python](./spotflow-py/CHANGELOG.md), [C](./spotflow-c/CHANGELOG.md)) with a brief description of the changes. The changelog follows the [Keep a Changelog](https://keepachangelog.com) format.
