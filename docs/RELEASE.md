# Release process

Releases are **tag-driven**. Pushing a tag `v*` (e.g. `v0.1.3`) triggers:

1. **Docker workflow** (`.github/workflows/docker.yml`)
   - Builds and pushes operator image using version from `crates/openapi-k8s-operator/Cargo.toml`
   - Tags: `OPERATOR_VERSION`, tag version (e.g. `0.1.3`), `latest`

2. **Release workflow** (`.github/workflows/release.yml`)
   - Sets Helm Chart `version` from the git tag and `appVersion` from operator Cargo.toml
   - Generates `CHANGELOG.md` via **cocogitto** (from conventional commits)
   - Packages the chart and updates the Helm index on `gh-pages`
   - Creates the GitHub release with the generated changelog

## Before tagging

- [ ] Version in `crates/openapi-k8s-operator/Cargo.toml` matches the release (e.g. `0.1.3`)
- [ ] `helm/openapi-k8s-operator/Chart.yaml` has `appVersion` aligned with operator version (workflow overwrites at release, but keep in sync)
- [ ] Changes are merged to the default branch (e.g. `master`)

## To release

```bash
# From default branch with release commit merged
git tag v0.1.3
git push origin v0.1.3
```

The workflows run automatically. The GitHub release and Helm chart index will be updated.
