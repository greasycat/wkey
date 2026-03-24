# Repo Notes

## CI / Test Environment
- GitHub Actions has incomplete `fzf` and Unicode terminal support.
- Tests that depend on launching `fzf` should continue to guard on `GITHUB_ACTIONS` so local behavior is covered without making CI flaky.

