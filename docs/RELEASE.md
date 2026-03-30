# Release process

Releases are **tag-driven**. Pushing a tag `v*` (e.g. `v0.1.3`) triggers:

1. **Docker workflow** (`.github/workflows/docker.yml`)
   - Operator image: version from `crates/openapi-k8s-operator/Cargo.toml`
   - Doc server image: version from `crates/openapi-doc-server/Cargo.toml` (independent semver)
   - Tags: crate version, git tag version, `latest`

2. **Release workflow** (`.github/workflows/release.yml`)
   - Sets Helm Chart `version` from the git tag and `appVersion` from operator `Cargo.toml` (not the doc server)
   - Generates `CHANGELOG.md` via **cocogitto** (from conventional commits)
   - Packages the chart and updates the Helm index on `gh-pages`
   - Creates the GitHub release with the generated changelog

## Before tagging

- [ ] Operator: `crates/openapi-k8s-operator/Cargo.toml`, `openapi-common` (workspace version), and `operator.image.tag` in `values.yaml` align
- [ ] Doc server: `crates/openapi-doc-server/Cargo.toml` and `openapiServer.image.tag` in `values.yaml` align (bump independently of the operator)
- [ ] `helm/openapi-k8s-operator/Chart.yaml` `appVersion` matches the operator crate (release overwrites chart `version` from the git tag)
- [ ] Changes are merged to the default branch (e.g. `master`)

## To release

```bash
# From default branch with release commit merged
git tag v0.1.3
git push origin v0.1.3
```

The workflows run automatically. The GitHub release and Helm chart index will be updated.
