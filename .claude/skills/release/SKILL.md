---
name: release
description: Version bump, changelog update, git tag, and push
disable-model-invocation: true
---

# Release

Create a new release — bump version, update changelog, tag.

## Steps
1. Ensure working tree is clean and all tests pass
2. Update version in `Cargo.toml` (`[workspace.package] version`)
3. Update `CHANGELOG.md` with new entries
4. Commit: `git commit -am "release: vX.Y.Z"`
5. Tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
6. Push: `git push && git push --tags`

## Version scheme
Follow [SemVer](https://semver.org/):
- MAJOR: breaking API changes
- MINOR: new features (backward compatible)
- PATCH: bug fixes

## Post-release
- Verify Docker image built on CI
- Check Vercel deployment
- Monitor error rate for 30 minutes
