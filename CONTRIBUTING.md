# Pull Request Notice

* Make sure each of your commits have a decent commit message.
* Make the PR against the `master` branch, unless it is a version bump for the `stable_release` git submodule.
* Run `cargo fmt` and `cargo clippy`

Please avoid sending a pull request with recursive merge nodes, as they
are impossible to fix once merged. Please rebase your branch on
jean-airoldie/zeromq-src-rs master instead of merging it.

```
git remote add upstream git@github.com:jean-airoldie/zeromq-src-rs.git
git fetch upstream
git rebase upstream/master
git push -f
```
