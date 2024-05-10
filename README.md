# git-dagger

`git dagger` generates a random Git commit graph (which is a directed acyclic graph (DAG)).

```
usage: git dagger [--count=<value>] [--linearity=<value>]

--count=<value>        number of commits to generate (default: 30)
--linearity=<value>    linearity factor (default: 0.0)
```

The generated graph has a single "terminal" commit (a commit which all other generated commits are ancestors of); this commit's hash is printed to stdout. The generated graph is likely to have multiple initial commits.

The graph structure is generated starting from the "terminal commit" node, iteratively adding new commit nodes that are parented to previously generated nodes. The linearity factor determines the connectivity of the graph. At zero, the last commit to be generated has a ~50% chance to be a parent of every other commit, and the number of graph connections grows quadratically with the commit count. At positive values, the probability that a particular previously generated commit receives a new parent decreases exponentially with every newly generated commit, and the number of graph connections grows linearly with the commit count. The linearity factor controls the strength of this exponential decay.

The generated commits are all empty.

This program doesn't serve much of a practical purpose. It's interesting to see `git log --oneline --graph` attempt to visualize the incomprehensible commit spaghetti it produces, though. And with a linearity factor of zero, scrolling through this visualization tends to crash MinTTY on my computer, so perhaps this program can find some use in stress testing.
