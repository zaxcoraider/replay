## Summary

<!-- What does this PR do? 1-3 bullet points. -->

- 

## Test plan

<!-- How did you verify this works? -->

- [ ] `cargo test -p replay-core --lib` passes
- [ ] `cargo test -p replay-api` passes
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cd web && pnpm tsc --noEmit` clean (if web changed)
- [ ] Manually tested against a real tx signature (if touching replay logic)

## Related issues

Closes #
