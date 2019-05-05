# graft

## Description
Replicate the structure of an original directory-tree and populate the replica with symlinks to the original directory-tree.

This is useful when the original directory-tree is not writable by the user.

It handles the case where the original directory-tree has a relative symlink within the tree.

## Usage
```
  $ mkdir $TOP_OF_REPLICA
  $ cd $TOP_OF_REPLICA
  $ graft $TOP_OF_ORIGINAL
```

## Compile
```
  $ cargo build --release
```
