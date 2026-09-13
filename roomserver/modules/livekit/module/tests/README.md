# Ignored LiveKit Tests

At the time of writing these tests, the GitLab pipeline didn't expose the
docker socket. Tests using `testcontainer` therefore failed in the pipeline.
These tests are therefore ignored. You can still run them locally using:

```bash
cargo test -p opentalk-roomserver-module-livekit -- --ignored
```
